//! The conversation state machines exercised end to end, including
//! the semantics the spec calls out: first valid answer wins, stale
//! answers are ignored, validation keeps the turn open for everyone,
//! late joiners receive current state.

use serde_json::{json, Map, Value};

use msip::daemon::{Conversation, Disposition, TurnSpec};
use msip::element::{types, Element};
use msip::frame::MsgType;
use msip::msg::{Outcome, Turn, Update};
use msip::surface::{Event, Session};

fn val(v: Value) -> Value {
    v
}

fn disk_turn() -> TurnSpec {
    let mut intro = Element::new("disk.intro", types::TEXT);
    intro.state.insert("text".into(), json!("Everything on the selected disk will be erased."));
    let mut target = Element::new("disk.target", types::SELECT);
    target.name = Some("Target disk".into());
    target.required = true;
    target.class = vec!["disk.select".into()];
    target.state.insert(
        "choices".into(),
        json!([{"value": "/dev/sda", "name": "QEMU HARDDISK — 32 GiB"}]),
    );
    let mut back = Element::new("nav.back", types::ACTION);
    back.state.insert("validate".into(), json!(false));
    let mut next = Element::new("nav.next", types::ACTION);
    next.state.insert("primary".into(), json!(true));
    TurnSpec {
        id: Some("disk.choose".into()),
        name: Some("Choose a target disk".into()),
        elements: vec![intro, target, back, next],
        ..Default::default()
    }
}

fn broadcast_turn(surfaces: &mut [&mut Session], turn: &Turn) {
    for s in surfaces {
        let body = serde_json::to_value(turn).unwrap();
        assert!(matches!(s.handle(MsgType::Turn, body).unwrap(), Event::NewTurn));
    }
}

fn broadcast_update(surfaces: &mut [&mut Session], upd: &Update) {
    for s in surfaces {
        let body = serde_json::to_value(upd).unwrap();
        assert!(matches!(s.handle(MsgType::Update, body).unwrap(), Event::Updated));
    }
}

#[test]
fn first_valid_answer_wins_and_stale_is_ignored() {
    let mut conv = Conversation::new("install").unwrap();
    let mut a = Session::new();
    let mut b = Session::new();

    let turn = conv.issue(disk_turn()).unwrap().clone();
    broadcast_turn(&mut [&mut a, &mut b], &turn);

    // A answers first.
    let mut values = Map::new();
    values.insert("disk.target".into(), json!("/dev/sda"));
    let ans_a = a.answer(Some("nav.next"), values).unwrap();
    let Disposition::Valid(v) = conv.handle_answer(&ans_a) else {
        panic!("A's answer should be valid");
    };
    assert_eq!(v.values["disk.target"], json!("/dev/sda"));

    // The daemon accepts by issuing the next turn.
    let confirm = conv
        .issue(TurnSpec {
            id: Some("confirm".into()),
            elements: vec![Element::new("go", types::ACTION)],
            ..Default::default()
        })
        .unwrap()
        .clone();

    // B's answer to the old turn arrives late: stale, silently ignored.
    let mut values = Map::new();
    values.insert("disk.target".into(), json!("/dev/sda"));
    let ans_b = b.answer(Some("nav.next"), values).unwrap();
    assert!(matches!(conv.handle_answer(&ans_b), Disposition::Stale));

    // Both surfaces move on with the broadcast.
    broadcast_turn(&mut [&mut a, &mut b], &confirm);
    assert_eq!(a.page().unwrap().turn.seq, 2);
    assert_eq!(b.page().unwrap().turn.seq, 2);
}

#[test]
fn validation_keeps_the_turn_open_for_everyone() {
    let mut conv = Conversation::new("install").unwrap();
    let mut a = Session::new();
    let mut b = Session::new();

    let turn = conv.issue(disk_turn()).unwrap().clone();
    broadcast_turn(&mut [&mut a, &mut b], &turn);

    // A presses Next without choosing a disk: required is missing.
    let ans = a.answer(Some("nav.next"), Map::new()).unwrap();
    let Disposition::Invalid(missing) = conv.handle_answer(&ans) else {
        panic!("expected required-field failure");
    };
    assert_eq!(missing, vec![("disk.target".into(), "required".into())]);

    // The daemon rejects; every surface sees the error; the turn stays open.
    let upd = conv.reject(&missing).unwrap();
    broadcast_update(&mut [&mut a, &mut b], &upd);
    assert_eq!(
        b.page().unwrap().element("disk.target").unwrap().error.as_deref(),
        Some("required")
    );

    // Back is validate:false — it must sail through with the field still empty.
    let ans = b.answer(Some("nav.back"), Map::new()).unwrap();
    let Disposition::Valid(v) = conv.handle_answer(&ans) else {
        panic!("back must skip validation");
    };
    assert_eq!(v.action.as_deref(), Some("nav.back"));

    // B can also still answer properly: the turn never closed.
    let mut values = Map::new();
    values.insert("disk.target".into(), json!("/dev/sda"));
    let ans = b.answer(Some("nav.next"), values).unwrap();
    assert!(matches!(conv.handle_answer(&ans), Disposition::Valid(_)));
}

#[test]
fn updates_fold_into_the_stored_turn_for_late_joiners() {
    let mut conv = Conversation::new("install").unwrap();
    let mut copy = Element::new("copy", types::PROGRESS);
    copy.state.insert("value".into(), json!(0));
    copy.state.insert("max".into(), json!(100));
    let mut log = Element::new("out", types::LOG);
    log.state.insert("lines".into(), json!([]));
    conv.issue(TurnSpec {
        id: Some("install.progress".into()),
        elements: vec![copy, log],
        ..Default::default()
    })
    .unwrap();

    // Progress advances by sparse update.
    let mut patch = Map::new();
    patch.insert("ref".into(), json!("copy"));
    patch.insert("value".into(), json!(42));
    conv.update(vec![patch]).unwrap();

    // A patch naming an unknown ref is refused.
    let mut bad = Map::new();
    bad.insert("ref".into(), json!("nope"));
    assert!(conv.update(vec![bad]).is_err());

    // A surface attaching now receives the *current* state.
    let mut late = Session::new();
    let body = serde_json::to_value(conv.current_turn().unwrap()).unwrap();
    late.handle(MsgType::Turn, body).unwrap();
    assert_eq!(
        late.page().unwrap().element("copy").unwrap().state["value"],
        json!(42)
    );

    // No inputs, no actions: the daemon advances on its own.
    let end = conv.end(Outcome::Complete, None).unwrap();
    assert_eq!(end.outcome, Outcome::Complete);
    assert!(conv.issue(disk_turn()).is_err());
}

#[test]
fn answer_arbitration_rejects_protocol_violations() {
    let mut conv = Conversation::new("install").unwrap();
    let turn = conv.issue(disk_turn()).unwrap().clone();

    let answer = |action: Option<&str>, values: Vec<(&str, Value)>| msip::msg::Answer {
        seq: turn.seq,
        action: action.map(str::to_string),
        values: values
            .into_iter()
            .map(|(k, v)| (k.to_string(), val(v)))
            .collect(),
    };

    // Unknown value ref.
    let d = conv.handle_answer(&answer(Some("nav.next"), vec![("nope", json!(1))]));
    assert!(matches!(d, Disposition::Protocol(_)));
    // Value on an action element.
    let d = conv.handle_answer(&answer(Some("nav.next"), vec![("nav.back", json!(1))]));
    assert!(matches!(d, Disposition::Protocol(_)));
    // Naming a non-action as the action.
    let d = conv.handle_answer(&answer(Some("disk.target"), vec![]));
    assert!(matches!(d, Disposition::Protocol(_)));
    // Omitting the action when the turn offers actions.
    let d = conv.handle_answer(&answer(None, vec![]));
    assert!(matches!(d, Disposition::Protocol(_)));
}

#[test]
fn defaults_fill_unanswered_optional_elements() {
    let mut conv = Conversation::new("oobe").unwrap();
    let mut hostname = Element::new("hostname", types::STRING);
    hostname.default = Some(json!("peios-4f2a"));
    conv.issue(TurnSpec {
        elements: vec![hostname, Element::new("ok", types::ACTION)],
        ..Default::default()
    })
    .unwrap();

    let ans = msip::msg::Answer {
        seq: 1,
        action: Some("ok".into()),
        values: Map::new(),
    };
    let Disposition::Valid(v) = conv.handle_answer(&ans) else {
        panic!()
    };
    assert_eq!(v.values["hostname"], json!("peios-4f2a"));
}

#[test]
fn log_append_folds_into_lines_on_both_sides() {
    let mut conv = Conversation::new("install").unwrap();
    let mut log = Element::new("out", types::LOG);
    log.state.insert("lines".into(), json!(["first"]));
    conv.issue(TurnSpec { elements: vec![log], ..Default::default() }).unwrap();

    let mut surf = Session::new();
    let body = serde_json::to_value(conv.current_turn().unwrap()).unwrap();
    surf.handle(MsgType::Turn, body).unwrap();

    let mut patch = Map::new();
    patch.insert("ref".into(), json!("out"));
    patch.insert("append".into(), json!(["second", "third"]));
    let upd = conv.update(vec![patch]).unwrap();
    let body = serde_json::to_value(&upd).unwrap();
    surf.handle(MsgType::Update, body).unwrap();

    let expect = json!(["first", "second", "third"]);
    // Folded on the daemon side (what a late joiner is sent)…
    assert_eq!(
        conv.current_turn().unwrap().elements[0].state["lines"], expect
    );
    assert!(!conv.current_turn().unwrap().elements[0].state.contains_key("append"));
    // …and on the surface side (what a renderer draws).
    assert_eq!(surf.page().unwrap().element("out").unwrap().state["lines"], expect);
}
