//! Built-in encodings are checked before application-specific validation.

use serde_json::{Value, json};

use msip::daemon::{Conversation, Disposition, TurnSpec};
use msip::element::{Element, types};
use msip::msg::{Answer, ErrorCode};

fn answer(element: Element, value: Option<Value>) -> Disposition {
    let mut conversation = Conversation::new("test").unwrap();
    conversation
        .issue(TurnSpec {
            elements: vec![element],
            ..Default::default()
        })
        .unwrap();
    let values = value
        .into_iter()
        .map(|value| ("input".into(), value))
        .collect();
    conversation.handle_answer(&Answer {
        seq: 1,
        action: None,
        values,
    })
}

fn assert_bad_value(disposition: Disposition) {
    assert!(
        matches!(disposition, Disposition::Protocol(error) if error.code == ErrorCode::BadValue)
    );
}

#[test]
fn outputs_never_accept_answers_or_require_values() {
    for kind in [types::TEXT, types::PROGRESS, types::LOG] {
        let mut element = Element::new("input", kind);
        element.required = true;
        element.default = Some(json!("must not be inserted"));
        assert_bad_value(answer(element.clone(), Some(json!("value"))));
        let Disposition::Valid(valid) = answer(element, None) else {
            panic!("output required a value")
        };
        assert!(valid.values.is_empty());
    }
}

#[test]
fn built_in_scalar_encodings_are_checked() {
    for (kind, valid, invalid) in [
        (types::STRING, json!("text"), json!(true)),
        (types::BOOLEAN, json!(false), json!("false")),
    ] {
        let element = Element::new("input", kind);
        assert!(matches!(
            answer(element.clone(), Some(valid)),
            Disposition::Valid(_)
        ));
        assert_bad_value(answer(element.clone(), Some(invalid)));
        assert_bad_value(answer(element, Some(Value::Null)));
    }
}

#[test]
fn select_answers_must_use_offered_values() {
    let mut element = Element::new("input", types::SELECT);
    element.state.insert(
        "choices".into(),
        json!([{"value": "one"}, {"value": {"id": 2}}]),
    );
    assert!(matches!(
        answer(element.clone(), Some(json!({"id": 2}))),
        Disposition::Valid(_)
    ));
    assert_bad_value(answer(element.clone(), Some(json!("missing"))));
    element.state.insert("multiple".into(), json!(true));
    assert!(matches!(
        answer(element.clone(), Some(json!(["one", {"id": 2}]))),
        Disposition::Valid(_)
    ));
    assert!(matches!(
        answer(element.clone(), Some(json!([]))),
        Disposition::Valid(_)
    ));
    assert_bad_value(answer(element.clone(), Some(json!("one"))));
    assert_bad_value(answer(element, Some(json!(["one", "missing"]))));
}

#[test]
fn table_answers_must_select_enabled_rows() {
    let mut element = Element::new("input", types::TABLE);
    element.state.insert(
        "rows".into(),
        json!([
            {"value": 1}, {"value": 2, "enabled": true}, {"value": 3, "enabled": false}
        ]),
    );
    for value in [1, 2] {
        assert!(matches!(
            answer(element.clone(), Some(json!(value))),
            Disposition::Valid(_)
        ));
    }
    for value in [3, 4] {
        assert_bad_value(answer(element.clone(), Some(json!(value))));
    }
}

#[test]
fn extensions_are_left_to_the_application_and_disabled_inputs_are_rejected() {
    let mut element = Element::new("input", "vendor.widget");
    assert!(matches!(
        answer(element.clone(), Some(json!({"custom": [1]}))),
        Disposition::Valid(_)
    ));
    element.enabled = false;
    assert_bad_value(answer(element, Some(json!("disabled"))));
}

#[test]
fn nonvalidating_action_still_checks_supplied_input_encodings() {
    let mut conversation = Conversation::new("test").unwrap();
    let mut action = Element::new("back", types::ACTION);
    action.state.insert("validate".into(), json!(false));
    let mut input = Element::new("input", types::STRING);
    input.required = true;
    input.default = Some(json!("default"));
    conversation
        .issue(TurnSpec {
            elements: vec![input, action],
            ..Default::default()
        })
        .unwrap();
    let mut answer = Answer {
        seq: 1,
        action: Some("back".into()),
        values: Default::default(),
    };
    assert!(matches!(
        conversation.handle_answer(&answer),
        Disposition::Valid(_)
    ));
    answer.values.insert("input".into(), json!(false));
    assert_bad_value(conversation.handle_answer(&answer));
}

#[test]
fn required_values_are_explicit_even_when_a_default_exists() {
    let mut element = Element::new("input", types::STRING);
    element.default = Some(json!("default"));
    element.required = true;
    let Disposition::Invalid(missing) = answer(element.clone(), None) else {
        panic!("required input accepted an implicit default");
    };
    assert_eq!(missing, vec![("input".into(), "required".into())]);
    assert!(matches!(
        answer(element.clone(), Some(json!("explicit"))),
        Disposition::Valid(_)
    ));
    element.required = false;
    let Disposition::Valid(valid) = answer(element, None) else {
        panic!("optional default was rejected");
    };
    assert_eq!(valid.values["input"], json!("default"));
}
