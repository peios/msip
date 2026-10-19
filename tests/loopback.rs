//! A scripted daemon and a surface talking real frames over a real
//! socket pair: HELLO/WELCOME, START/BOUND, two turns, an answer, a
//! progress update, END.

use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

use serde_json::{Map, json};

use msip::daemon::{Conversation, Disposition, TurnSpec};
use msip::element::{Element, types};
use msip::frame::{FrameError, MsgType, read_msg, write_msg};
use msip::msg::{Bound, Hello, Outcome, Welcome};
use msip::surface::{Event, Session};

#[test]
fn full_conversation_over_a_socket() {
    let (mut d_sock, mut s_sock) = UnixStream::pair().unwrap();

    for socket in [&d_sock, &s_sock] {
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
    }
    let daemon = thread::spawn(move || run_daemon(&mut d_sock));

    // The surface side.
    let mut session = Session::new();
    write_msg(
        &mut s_sock,
        MsgType::Hello,
        &Hello {
            surface: Some("test/0".into()),
            element_types: types::ALL.iter().map(|s| s.to_string()).collect(),
        },
    )
    .unwrap();
    let (t, body) = read_msg(&mut s_sock).unwrap();
    let Event::Welcome(w) = session.handle(t, body).unwrap() else {
        panic!()
    };
    assert_eq!(w.kinds, vec!["install"]);

    write_msg(
        &mut s_sock,
        MsgType::Start,
        &msip::msg::Start {
            kind: "install".into(),
        },
    )
    .unwrap();
    let (t, body) = read_msg(&mut s_sock).unwrap();
    let Event::Bound(b) = session.handle(t, body).unwrap() else {
        panic!()
    };
    assert_eq!(b.conversation.len(), 32);

    let (t, body) = read_msg(&mut s_sock).unwrap();
    assert!(matches!(session.handle(t, body).unwrap(), Event::NewTurn));
    let mut values = Map::new();
    values.insert("disk.target".into(), json!("/dev/sda"));
    let ans = session.answer(Some("nav.next"), values).unwrap();
    write_msg(&mut s_sock, MsgType::Answer, &ans).unwrap();

    let (t, body) = read_msg(&mut s_sock).unwrap();
    assert!(matches!(session.handle(t, body).unwrap(), Event::NewTurn));
    let (t, body) = read_msg(&mut s_sock).unwrap();
    assert!(matches!(session.handle(t, body).unwrap(), Event::Updated));
    assert_eq!(
        session.page().unwrap().element("copy").unwrap().state["value"],
        json!(100)
    );
    let (t, body) = read_msg(&mut s_sock).unwrap();
    let Event::Ended(end) = session.handle(t, body).unwrap() else {
        panic!()
    };
    assert_eq!(end.outcome, Outcome::Complete);

    daemon.join().unwrap();
}

fn run_daemon(socket: &mut UnixStream) {
    // HELLO -> WELCOME
    let (t, body) = read_msg(socket).unwrap();
    assert_eq!(t, MsgType::Hello);
    let hello: Hello = serde_json::from_value(body).unwrap();
    assert!(hello.element_types.iter().any(|t| t == types::SELECT));
    write_msg(
        socket,
        MsgType::Welcome,
        &Welcome {
            daemon: Some("installerd/0.0".into()),
            kinds: vec!["install".into()],
        },
    )
    .unwrap();

    // START -> BOUND + first TURN
    let (t, body) = read_msg(socket).unwrap();
    assert_eq!(t, MsgType::Start);
    assert_eq!(body["kind"], json!("install"));
    let mut conv = Conversation::new("install").unwrap();
    write_msg(
        socket,
        MsgType::Bound,
        &Bound {
            conversation: conv.id().to_string(),
            seq: conv.seq(),
            attached: false,
        },
    )
    .unwrap();
    send_disk_turn(socket, &mut conv);
    answer_and_complete(socket, &mut conv);
}

fn send_disk_turn(socket: &mut UnixStream, conversation: &mut Conversation) {
    let mut target = Element::new("disk.target", types::SELECT);
    target.required = true;
    target.state.insert(
        "choices".into(),
        json!([{"value": "/dev/sda", "name": "disk"}]),
    );
    let turn = conversation
        .issue(TurnSpec {
            id: Some("disk.choose".into()),
            elements: vec![target, Element::new("nav.next", types::ACTION)],
            ..Default::default()
        })
        .unwrap()
        .clone();
    write_msg(socket, MsgType::Turn, &turn).unwrap();
}

fn answer_and_complete(socket: &mut UnixStream, conversation: &mut Conversation) {
    // ANSWER -> next TURN (progress), one UPDATE, END
    let (t, body) = read_msg(socket).unwrap();
    assert_eq!(t, MsgType::Answer);
    let ans = serde_json::from_value(body).unwrap();
    let Disposition::Valid(v) = conversation.handle_answer(&ans) else {
        panic!("expected a valid answer")
    };
    assert_eq!(v.values["disk.target"], json!("/dev/sda"));

    let mut prog = Element::new("copy", types::PROGRESS);
    prog.state.insert("value".into(), json!(0));
    prog.state.insert("max".into(), json!(100));
    let turn = conversation
        .issue(TurnSpec {
            id: Some("install.progress".into()),
            elements: vec![prog],
            ..Default::default()
        })
        .unwrap()
        .clone();
    write_msg(socket, MsgType::Turn, &turn).unwrap();
    let mut patch = Map::new();
    patch.insert("ref".into(), json!("copy"));
    patch.insert("value".into(), json!(100));
    let upd = conversation.update(vec![patch]).unwrap();
    write_msg(socket, MsgType::Update, &upd).unwrap();
    let end = conversation.end(Outcome::Complete, None).unwrap();
    write_msg(socket, MsgType::End, &end).unwrap();
}

#[test]
fn wrong_magic_rejects_the_frame() {
    let mut bytes = &b"PGSL\x01\x00\x01\x00\x0c\x00\x00\x00"[..];
    match read_msg(&mut bytes) {
        Err(FrameError::BadMagic(magic)) => assert_eq!(&magic, b"PGSL"),
        other => panic!("expected BadMagic, got {other:?}"),
    }
}

#[test]
fn oversized_declaration_is_rejected_without_reading_the_body() {
    let mut header = Vec::new();
    header.extend_from_slice(b"MSIP");
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&(2u32 << 20).to_le_bytes());
    assert!(matches!(
        read_msg(&mut header.as_slice()),
        Err(FrameError::TooLarge(_))
    ));
}
