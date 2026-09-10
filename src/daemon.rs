//! The daemon side of a conversation.
//!
//! [`Conversation`] is the turn state machine: it assigns sequence
//! numbers, remembers the outstanding page (patched by every update it
//! issues, so a late joiner receives current state, not history), and
//! arbitrates answers — first valid answer wins, stale answers are
//! ignored, malformed ones are protocol errors. It is transport-free:
//! the embedding daemon owns sockets, broadcast fan-out and access
//! control, and sends whatever this type returns to every attached
//! surface.

use std::fmt;
use std::fs::File;
use std::io::{self, Read};

use serde_json::{Map, Value};

use crate::element::{Element, types};
use crate::msg::{Answer, End, ErrorCode, ErrorMsg, Outcome, Turn, Update};

/// A turn as the application describes it; `Conversation::issue`
/// assigns the seq.
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

#[derive(Debug)]
pub enum StateError {
    /// The ref named in a sparse update does not exist in the
    /// outstanding turn.
    UnknownRef(String),
    /// No turn is outstanding.
    NoTurn,
    /// The conversation has ended.
    Ended,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::UnknownRef(r) => write!(f, "no element {r:?} in the outstanding turn"),
            StateError::NoTurn => write!(f, "no turn outstanding"),
            StateError::Ended => write!(f, "conversation has ended"),
        }
    }
}

impl std::error::Error for StateError {}

pub struct Conversation {
    id: String,
    kind: String,
    seq: u64,
    current: Option<Turn>,
    ended: bool,
}

impl Conversation {
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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

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
    pub fn update(&mut self, patches: Vec<Map<String, Value>>) -> Result<Update, StateError> {
        if self.ended {
            return Err(StateError::Ended);
        }
        let turn = self.current.as_mut().ok_or(StateError::NoTurn)?;
        for patch in &patches {
            let target = patch.get("ref").and_then(Value::as_str).unwrap_or_default();
            let elem = turn
                .elements
                .iter_mut()
                .find(|e| e.r#ref == target)
                .ok_or_else(|| StateError::UnknownRef(target.to_string()))?;
            merge_patch(elem, patch);
        }
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
            .map(|e| match serde_json::to_value(e) {
                Ok(Value::Object(m)) => m,
                _ => Map::new(),
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
    /// required-field presence, default filling — and leaves domain
    /// validation to the application.
    pub fn handle_answer(&self, ans: &Answer) -> Disposition {
        if self.ended {
            return Disposition::Stale;
        }
        let turn = match &self.current {
            Some(t) if t.seq == ans.seq => t,
            _ => return Disposition::Stale,
        };

        let element = |r: &str| turn.elements.iter().find(|e| e.r#ref == r);

        // Values must name enabled, non-action elements of the turn.
        for r in ans.values.keys() {
            match element(r) {
                None => {
                    return Disposition::Protocol(ErrorMsg {
                        code: ErrorCode::UnknownRef,
                        message: Some(format!("no element {r:?} in turn {}", turn.seq)),
                    });
                }
                Some(e) if e.is_action() => {
                    return Disposition::Protocol(ErrorMsg {
                        code: ErrorCode::BadValue,
                        message: Some(format!("{r:?} is an action and carries no value")),
                    });
                }
                Some(e) if !e.enabled => {
                    return Disposition::Protocol(ErrorMsg {
                        code: ErrorCode::BadValue,
                        message: Some(format!("{r:?} is disabled")),
                    });
                }
                Some(_) => {}
            }
        }

        // The action must be an enabled action element; a turn that
        // offers actions must have one named, and one that offers none
        // takes the implicit submit (action absent).
        let has_actions = turn.elements.iter().any(|e| e.is_action() && e.enabled);
        let validating = match &ans.action {
            Some(a) => match element(a) {
                Some(e) if e.is_action() && e.enabled => e.validates(),
                Some(_) => {
                    return Disposition::Protocol(ErrorMsg {
                        code: ErrorCode::BadValue,
                        message: Some(format!("{a:?} is not an enabled action")),
                    });
                }
                None => {
                    return Disposition::Protocol(ErrorMsg {
                        code: ErrorCode::UnknownRef,
                        message: Some(format!("no action {a:?} in turn {}", turn.seq)),
                    });
                }
            },
            None if has_actions => {
                return Disposition::Protocol(ErrorMsg {
                    code: ErrorCode::BadValue,
                    message: Some("turn offers actions; the answer must name one".into()),
                });
            }
            None => true,
        };

        let mut values = ans.values.clone();
        let mut missing = Vec::new();
        if validating {
            for e in &turn.elements {
                if e.is_action() || !e.enabled || e.r#type == types::TEXT {
                    continue;
                }
                if !values.contains_key(&e.r#ref) {
                    if let Some(d) = &e.default {
                        values.insert(e.r#ref.clone(), d.clone());
                    } else if e.required {
                        missing.push((e.r#ref.clone(), "required".to_string()));
                    }
                }
            }
        }
        if !missing.is_empty() {
            return Disposition::Invalid(missing);
        }
        Disposition::Valid(ValidAnswer {
            action: ans.action.clone(),
            values,
        })
    }
}

/// Overlay a sparse patch onto a stored element, keeping the typed
/// common fields and the open state map coherent.
fn merge_patch(elem: &mut Element, patch: &Map<String, Value>) {
    let mut as_value = match serde_json::to_value(&*elem) {
        Ok(Value::Object(m)) => m,
        _ => return,
    };
    let folded_append = fold_log_append(&mut as_value, patch);
    for (k, v) in patch {
        if k == "ref" || (folded_append && k == "append") {
            continue;
        }
        if v.is_null() {
            as_value.remove(k);
        } else {
            as_value.insert(k.clone(), v.clone());
        }
    }
    if let Ok(updated) = serde_json::from_value(Value::Object(as_value)) {
        *elem = updated;
    }
}

/// The one type-aware folding rule: a sparse patch to a `log` element
/// carrying `append` extends `lines` rather than storing `append`
/// (§3.B), so folded state accumulates and a late joiner receives the
/// whole log.
fn fold_log_append(
    as_value: &mut serde_json::Map<String, Value>,
    patch: &Map<String, Value>,
) -> bool {
    let is_log = as_value.get("type").and_then(Value::as_str) == Some("log");
    let Some(Value::Array(added)) = patch.get("append") else {
        return false;
    };
    if !is_log {
        return false;
    }
    let lines = as_value
        .entry("lines".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    if let Value::Array(lines) = lines {
        lines.extend(added.iter().cloned());
    }
    true
}
