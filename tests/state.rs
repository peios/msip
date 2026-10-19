//! A rejected message must leave the published page unchanged.

use serde_json::{Map, Value, json};

use msip::daemon::{Conversation, StateError, TurnSpec};
use msip::element::{Element, types};
use msip::frame::MsgType;
use msip::surface::{Event, Session};

fn setup() -> (Conversation, Session) {
    let mut conversation = Conversation::new("test").unwrap();
    let mut log = Element::new("log", types::LOG);
    log.state.insert("lines".into(), json!(["initial"]));
    conversation
        .issue(TurnSpec {
            elements: vec![log],
            ..Default::default()
        })
        .unwrap();
    let mut session = Session::new();
    session
        .handle(
            MsgType::Turn,
            serde_json::to_value(conversation.current_turn()).unwrap(),
        )
        .unwrap();
    (conversation, session)
}

fn object(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

#[test]
fn rejected_sparse_batches_are_atomic_on_daemon_and_surface() {
    for invalid in [
        json!({"ref": "unknown"}),
        json!({"ref": 1}),
        json!({"name": "missing ref"}),
        json!({"ref": "log", "type": null}),
        json!({"ref": "log", "enabled": "yes"}),
        json!({"ref": "log", "append": "not an array"}),
    ] {
        let (mut conversation, mut session) = setup();
        let before = conversation.current_turn().unwrap().elements.clone();
        let patches = vec![
            object(json!({"ref": "log", "append": ["must not appear"]})),
            object(invalid),
        ];
        assert!(conversation.update(patches.clone()).is_err());
        assert_eq!(conversation.current_turn().unwrap().elements, before);
        assert!(
            session
                .handle(MsgType::Update, json!({"seq": 1, "elements": patches}))
                .is_err()
        );
        assert_eq!(session.page().unwrap().turn.elements, before);
    }
}

#[test]
fn malformed_patch_is_distinct_from_unknown_ref() {
    let (mut conversation, _) = setup();
    assert!(matches!(
        conversation.update(vec![object(json!({"ref": 1}))]),
        Err(StateError::Malformed(_))
    ));
    assert!(matches!(
        conversation.update(vec![object(json!({"ref": "missing"}))]),
        Err(StateError::UnknownRef(_))
    ));
}

#[test]
fn repeated_refs_fold_in_order_and_null_removes_optional_fields() {
    let (mut conversation, mut session) = setup();
    let update = conversation
        .update(vec![
            object(json!({"ref": "log", "append": ["second"], "error": "old error"})),
            object(json!({"ref": "log", "append": ["third"], "error": null})),
        ])
        .unwrap();
    session
        .handle(MsgType::Update, serde_json::to_value(update).unwrap())
        .unwrap();
    let element = session.page().unwrap().element("log").unwrap();
    assert_eq!(
        element.state["lines"],
        json!(["initial", "second", "third"])
    );
    assert!(element.error.is_none());
    assert!(!element.state.contains_key("append"));
    assert_eq!(
        session.page().unwrap().turn.elements,
        conversation.current_turn().unwrap().elements
    );
}

#[test]
fn invalid_full_update_and_end_preserve_the_page() {
    let (_, mut session) = setup();
    let before = session.page().unwrap().turn.elements.clone();
    assert!(
        session
            .handle(
                MsgType::Update,
                json!({"seq": 1, "full": true, "elements": [
                    {"ref": "new", "type": "text"}, {"ref": "broken"}
                ]})
            )
            .is_err()
    );
    assert_eq!(session.page().unwrap().turn.elements, before);
    assert!(
        session
            .handle(MsgType::End, json!({"seq": 1, "outcome": "invalid"}))
            .is_err()
    );
    assert_eq!(session.page().unwrap().turn.elements, before);
    assert!(matches!(
        session
            .handle(MsgType::End, json!({"seq": 1, "outcome": "complete"}))
            .unwrap(),
        Event::Ended(_)
    ));
    assert!(session.page().is_none());
}

#[test]
fn stale_update_does_not_touch_the_page() {
    let (_, mut session) = setup();
    let before = session.page().unwrap().turn.elements.clone();
    session
        .handle(
            MsgType::Update,
            json!({"seq": 0, "elements": [{"ref": "missing"}]}),
        )
        .unwrap();
    assert_eq!(session.page().unwrap().turn.elements, before);
}
