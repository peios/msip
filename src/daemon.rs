//! The daemon side of a conversation.
//!
//! [`Conversation`] is the turn state machine: it assigns sequence
//! numbers, remembers the outstanding page (patched by every update it
//! issues, so a late joiner receives current state, not history), and
//! checks answers for staleness, required fields, and protocol violations.
//! The caller must serialize answer checking, domain validation, and acceptance
//! through `issue` or `end` to enforce first-valid-answer arbitration. It is transport-free:
//! the embedding daemon owns sockets, broadcast fan-out and access
//! control, and sends whatever this type returns to every attached
//! surface.

use std::fmt;
use std::fs::File;
use std::io::{self, Read};

use serde_json::{Map, Value};

use crate::element::{Element, types};
use crate::patch::{self, PatchError};
use crate::msg::{Answer, End, ErrorCode, ErrorMsg, Outcome, Turn, Update};

/// A turn supplied by the application; [`Conversation::issue`] assigns its sequence.
///
/// The caller supplies unique refs, valid defaults and state, and only element
/// types supported by every attached surface. Secrets must stay out of state.
#[derive(Debug, Clone, Default)]
pub struct TurnSpec {
    pub id: Option<String>,
    pub name: Option<String>,
    pub class: Vec<String>,
    pub elements: Vec<Element>,
}

/// What became of a received answer.
#[derive(Debug)]
pub enum Disposition {
    /// Not for the outstanding turn (or the conversation is over).
    /// Ignore it: the sender already has, or is about to receive, the
    /// broadcast that supersedes it.
    Stale,
    /// The answer violates the protocol; send this on the offending
    /// connection only.
    Protocol(ErrorMsg),
    /// Structurally sound but failed required-field validation. The
    /// application will usually pass these to [`Conversation::reject`]
    /// (possibly with findings of its own added).
    Invalid(Vec<(String, String)>),
    /// The answer is applicable. The application validates its domain
    /// rules and then either issues the next turn (acceptance) or
    /// calls [`Conversation::reject`].
    Valid(ValidAnswer),
}

/// An applicable answer: the pressed action (if any) and the values,
/// with element defaults filled in for anything not supplied.
#[derive(Debug, Clone)]
pub struct ValidAnswer {
    pub action: Option<String>,
    pub values: Map<String, Value>,
}

/// A refused conversation mutation. An error leaves the published turn unchanged.
#[derive(Debug)]
pub enum StateError {
    /// The ref named in a sparse update does not exist in the
    /// outstanding turn.
    UnknownRef(String),
    /// No turn is outstanding.
    NoTurn,
    /// An update contains an invalid ref field or produces invalid element fields.
    Malformed(String),
    /// The conversation has ended.
    Ended,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::UnknownRef(r) => write!(f, "no element {r:?} in the outstanding turn"),
            StateError::Malformed(message) => write!(f, "malformed update: {message}"),
            StateError::NoTurn => write!(f, "no turn outstanding"),
            StateError::Ended => write!(f, "conversation has ended"),
        }
    }
}

impl std::error::Error for StateError {}

/// Transport-independent turn state owned by the embedding daemon.
///
/// This type does not manage connections, authorization, binding, or broadcasts.
/// Serialize answer validation and acceptance when several surfaces can answer.
pub struct Conversation {
    id: String,
    kind: String,
    seq: u64,
    current: Option<Turn>,
    ended: bool,
}

impl Conversation {
    /// Create an unstarted conversation using 128 random bits from `/dev/urandom`.
    ///
    /// Returns an I/O error if randomness cannot be read. The identifier is not
    /// an authorization credential; the caller must authorize each attachment.
    pub fn new(kind: &str) -> io::Result<Conversation> {
        let mut raw = [0u8; 16];
        File::open("/dev/urandom")?.read_exact(&mut raw)?;
        let id = raw.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Conversation {
            id,
            kind: kind.to_string(),
            seq: 0,
            current: None,
            ended: false,
        })
    }

    /// Return the conversation identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return the application-defined conversation kind.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Return the last issued sequence number, or zero before the first turn.
    pub fn seq(&self) -> u64 {
        self.seq
    }

    /// Report whether `end` has completed this conversation.
    pub fn ended(&self) -> bool {
        self.ended
    }

    /// The outstanding turn in its current state — what a late joiner
    /// is sent after Bound.
    pub fn current_turn(&self) -> Option<&Turn> {
        self.current.as_ref()
    }

    /// Issue the next turn. Closes the outstanding one implicitly:
    /// answers to it become stale. Returns the message to broadcast.
    pub fn issue(&mut self, spec: TurnSpec) -> Result<&Turn, StateError> {
        if self.ended {
            return Err(StateError::Ended);
        }
        self.seq += 1;
        self.current = Some(Turn {
            seq: self.seq,
            id: spec.id,
            name: spec.name,
            class: spec.class,
            elements: spec.elements,
        });
        Ok(self.current.as_ref().unwrap())
    }

    /// Sparse update: each patch is an object with `ref` plus changed
    /// fields. Patches are folded into the stored turn so late joiners
    /// see current state. Returns the message to broadcast.
    ///
    /// The whole batch is atomic. A malformed patch or unknown ref changes nothing.
    /// Repeated refs observe preceding patches in the same batch.
    pub fn update(&mut self, patches: Vec<Map<String, Value>>) -> Result<Update, StateError> {
        if self.ended {
            return Err(StateError::Ended);
        }
        let turn = self.current.as_mut().ok_or(StateError::NoTurn)?;
        patch::apply(&mut turn.elements, &patches).map_err(|error| match error {
            PatchError::UnknownRef(name) => StateError::UnknownRef(name),
            PatchError::Malformed(message) => StateError::Malformed(message),
        })?;
        Ok(Update {
            seq: turn.seq,
            full: false,
            elements: patches,
        })
    }

    /// Full update: replace the element list (the only way the set of
    /// refs may change mid-turn). Returns the message to broadcast.
    pub fn update_full(&mut self, elements: Vec<Element>) -> Result<Update, StateError> {
        if self.ended {
            return Err(StateError::Ended);
        }
        let turn = self.current.as_mut().ok_or(StateError::NoTurn)?;
        turn.elements = elements;
        let full = turn
            .elements
            .iter()
            .map(|element| {
                let Value::Object(fields) =
                    serde_json::to_value(element).expect("Element fields are JSON-serializable")
                else {
                    unreachable!("Element serializes as an object");
                };
                fields
            })
            .collect();
        Ok(Update {
            seq: turn.seq,
            full: true,
            elements: full,
        })
    }

    /// Reject the outstanding turn's latest answer for validation:
    /// stamps the errors onto the elements and returns the sparse
    /// update to broadcast. The turn stays open.
    pub fn reject(&mut self, errors: &[(String, String)]) -> Result<Update, StateError> {
        let patches = errors
            .iter()
            .map(|(r, msg)| {
                let mut p = Map::new();
                p.insert("ref".into(), Value::String(r.clone()));
                p.insert("error".into(), Value::String(msg.clone()));
                p
            })
            .collect();
        self.update(patches)
    }

    /// End the conversation. Returns the message to broadcast; the
    /// embedding daemon drops the conversation afterwards.
    pub fn end(&mut self, outcome: Outcome, message: Option<String>) -> Result<End, StateError> {
        if self.ended {
            return Err(StateError::Ended);
        }
        self.ended = true;
        Ok(End {
            seq: self.seq,
            outcome,
            message,
        })
    }

    /// Arbitrate a received answer. Does the structural work the
    /// protocol defines — staleness, ref existence, action rules,
    /// built-in input encodings and choices, required fields, and defaults.
    /// Domain rules and negotiated extension validation belong to the application.
    ///
    /// Checking does not consume an answer. Accept it by issuing the next turn
    /// or ending the conversation before checking another competing answer.
    pub fn handle_answer(&self, answer: &Answer) -> Disposition {
        if self.ended {
            return Disposition::Stale;
        }
        let turn = match &self.current {
            Some(t) if t.seq == answer.seq => t,
            _ => return Disposition::Stale,
        };

        if let Err(error) = check_values(turn, answer) {
            return Disposition::Protocol(error);
        }
        let validating = match validating_action(turn, answer.action.as_deref()) {
            Ok(validating) => validating,
            Err(error) => return Disposition::Protocol(error),
        };

        let mut values = answer.values.clone();
        let mut missing = Vec::new();
        if validating {
            for e in &turn.elements {
                if e.is_action() || !e.enabled || is_output(e) {
                    continue;
                }
                if !values.contains_key(&e.r#ref) {
                    if e.required {
                        missing.push((e.r#ref.clone(), "required".to_string()));
                    } else if let Some(default) = &e.default {
                        values.insert(e.r#ref.clone(), default.clone());
                    }
                }
            }
        }
        if !missing.is_empty() {
            return Disposition::Invalid(missing);
        }
        Disposition::Valid(ValidAnswer {
            action: answer.action.clone(),
            values,
        })
    }
}

fn protocol_error(code: ErrorCode, message: String) -> ErrorMsg {
    ErrorMsg {
        code,
        message: Some(message),
    }
}

fn is_output(element: &Element) -> bool {
    matches!(
        element.r#type.as_str(),
        types::TEXT | types::PROGRESS | types::LOG
    )
}

fn check_values(turn: &Turn, answer: &Answer) -> Result<(), ErrorMsg> {
    for (name, value) in &answer.values {
        let element = turn
            .elements
            .iter()
            .find(|element| element.r#ref == *name)
            .ok_or_else(|| {
                protocol_error(
                    ErrorCode::UnknownRef,
                    format!("no element {name:?} in turn {}", turn.seq),
                )
            })?;
        if !element.enabled || element.is_action() || is_output(element) {
            return Err(protocol_error(
                ErrorCode::BadValue,
                format!("{name:?} is not an enabled input"),
            ));
        }
        if !valid_value(element, value) {
            return Err(protocol_error(
                ErrorCode::BadValue,
                format!("invalid answer encoding or choice for {name:?}"),
            ));
        }
    }
    Ok(())
}

fn valid_value(element: &Element, value: &Value) -> bool {
    match element.r#type.as_str() {
        types::STRING => value.is_string(),
        types::BOOLEAN => value.is_boolean(),
        types::SELECT => {
            let multiple = element
                .state
                .get("multiple")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if multiple {
                value.as_array().is_some_and(|values| {
                    values
                        .iter()
                        .all(|value| offered_choice(element, "choices", value))
                })
            } else {
                offered_choice(element, "choices", value)
            }
        }
        types::TABLE => offered_choice(element, "rows", value),
        // The negotiated extension's contract belongs to the embedding daemon.
        _ => true,
    }
}

fn offered_choice(element: &Element, field: &str, value: &Value) -> bool {
    element
        .state
        .get(field)
        .and_then(Value::as_array)
        .is_some_and(|choices| {
            choices.iter().any(|choice| {
                choice.get("value") == Some(value)
                    && (field != "rows"
                        || choice
                            .get("enabled")
                            .and_then(Value::as_bool)
                            .unwrap_or(true))
            })
        })
}

fn validating_action(turn: &Turn, action: Option<&str>) -> Result<bool, ErrorMsg> {
    let Some(name) = action else {
        if turn
            .elements
            .iter()
            .any(|element| element.is_action() && element.enabled)
        {
            return Err(protocol_error(
                ErrorCode::BadValue,
                "turn offers actions; the answer must name one".into(),
            ));
        }
        return Ok(true);
    };
    let element = turn
        .elements
        .iter()
        .find(|element| element.r#ref == name)
        .ok_or_else(|| {
            protocol_error(
                ErrorCode::UnknownRef,
                format!("no action {name:?} in turn {}", turn.seq),
            )
        })?;
    if !element.is_action() || !element.enabled {
        return Err(protocol_error(
            ErrorCode::BadValue,
            format!("{name:?} is not an enabled action"),
        ));
    }
    Ok(element.validates())
}
