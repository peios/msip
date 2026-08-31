//! A scripted daemon and a surface talking real frames over a real
//! socket pair: HELLO/WELCOME, START/BOUND, two turns, an answer, a
//! progress update, END.

use std::os::unix::net::UnixStream;
use std::thread;

use serde_json::{json, Map};

use msip::daemon::{Conversation, Disposition, TurnSpec};
use msip::element::{types, Element};
use msip::frame::{read_msg, write_msg, FrameError, MsgType};
use msip::msg::{Bound, Hello, Outcome, Welcome};
use msip::surface::{Event, Session};

#[test]
fn full_conversation_over_a_socket() {
    let (mut d_sock, mut s_sock) = UnixStream::pair().unwrap();

    let daemon = thread::spawn(move || {
        // HELLO -> WELCOME
        let (t, body) = read_msg(&mut d_sock).unwrap();
        assert_eq!(t, MsgType::Hello);
        let hello: Hello = serde_json::from_value(body).unwrap();
        assert!(hello.element_types.iter().any(|t| t == types::SELECT));
        write_msg(
            &mut d_sock,
            MsgType::Welcome,
            &Welcome {
                daemon: Some("installerd/0.0".into()),
                kinds: vec!["install".into()],
            },
        )
        .unwrap();

        // START -> BOUND + first TURN
        let (t, body) = read_msg(&mut d_sock).unwrap();
        assert_eq!(t, MsgType::Start);
        assert_eq!(body["kind"], json!("install"));
        let mut conv = Conversation::new("install").unwrap();
        write_msg(
            &mut d_sock,
            MsgType::Bound,
            &Bound {
                conversation: conv.id().to_string(),
                seq: conv.seq(),
                attached: false,
            },
        )
        .unwrap();
        let mut target = Element::new("disk.target", types::SELECT);
        target.required = true;
        target
            .state
            .insert("choices".into(), json!([{"value": "/dev/sda", "name": "disk"}]));
        let turn = conv
            .issue(TurnSpec {
                id: Some("disk.choose".into()),
                elements: vec![target, Element::new("nav.next", types::ACTION)],
                ..Default::default()
            })
            .unwrap()
            .clone();
        write_msg(&mut d_sock, MsgType::Turn, &turn).unwrap();

        // ANSWER -> next TURN (progress), one UPDATE, END
        let (t, body) = read_msg(&mut d_sock).unwrap();
        assert_eq!(t, MsgType::Answer);
        let ans = serde_json::from_value(body).unwrap();
        let Disposition::Valid(v) = conv.handle_answer(&ans) else {
            panic!("expected a valid answer")
        };
        assert_eq!(v.values["disk.target"], json!("/dev/sda"));

        let mut prog = Element::new("copy", types::PROGRESS);
        prog.state.insert("value".into(), json!(0));
        prog.state.insert("max".into(), json!(100));
        let turn = conv
            .issue(TurnSpec {
                id: Some("install.progress".into()),
                elements: vec![prog],
                ..Default::default()
            })
            .unwrap()
            .clone();
        write_msg(&mut d_sock, MsgType::Turn, &turn).unwrap();
        let mut patch = Map::new();
        patch.insert("ref".into(), json!("copy"));
        patch.insert("value".into(), json!(100));
        let upd = conv.update(vec![patch]).unwrap();
        write_msg(&mut d_sock, MsgType::Update, &upd).unwrap();
        let end = conv.end(Outcome::Complete, None).unwrap();
        write_msg(&mut d_sock, MsgType::End, &end).unwrap();
    });

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

    write_msg(&mut s_sock, MsgType::Start, &msip::msg::Start { kind: "install".into() }).unwrap();
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

#[test]
fn wrong_magic_fails_the_connection_on_byte_zero() {
    use std::io::Write;
    let (mut a, mut b) = UnixStream::pair().unwrap();
    a.write_all(b"PGSL\x01\x00\x01\x00\x0c\x00\x00\x00").unwrap();
    match read_msg(&mut b) {
        Err(FrameError::BadMagic(m)) => assert_eq!(&m, b"PGSL"),
        other => panic!("expected BadMagic, got {other:?}"),
    }
}

#[test]
fn oversized_declaration_is_rejected_without_reading_the_body() {
    use std::io::Write;
    let (mut a, mut b) = UnixStream::pair().unwrap();
    let mut hdr = Vec::new();
    hdr.extend_from_slice(b"MSIP");
    hdr.extend_from_slice(&1u16.to_le_bytes());
    hdr.extend_from_slice(&1u16.to_le_bytes());
    hdr.extend_from_slice(&(2u32 << 20).to_le_bytes());
    a.write_all(&hdr).unwrap();
    assert!(matches!(read_msg(&mut b), Err(FrameError::TooLarge(_))));
}
