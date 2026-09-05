//! The element model. A turn is an ordered list of elements; an
//! element is identified by its `ref` (unique within the turn, stable
//! across turns and runs), carries a `type` from the negotiated set,
//! and beyond the common fields here its state is type-specific and
//! open-ended (`state`).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The element types this crate knows. The set is capability-negotiated
/// in HELLO (a statement of capability, not an enumeration a peer must
/// exhaust), so `Element::type` is an open string, not an enum.
pub mod types {
    pub const TEXT: &str = "text";
    pub const STRING: &str = "string";
    pub const BOOLEAN: &str = "boolean";
    pub const SELECT: &str = "select";
    pub const PROGRESS: &str = "progress";
    pub const LOG: &str = "log";
    pub const ACTION: &str = "action";
    /// Rows with columns, and a choice among them: a `select` whose
    /// choices have more to say than a name.
    pub const TABLE: &str = "table";

    pub const ALL: &[&str] = &[TEXT, STRING, BOOLEAN, SELECT, PROGRESS, LOG, ACTION, TABLE];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub r#ref: String,
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Presentation hint only; promises nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// No echo, never logged, and never carried in UPDATE state.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub secret: bool,
    #[serde(default = "yes", skip_serializing_if = "Clone::clone")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Type-specific state: `choices`, `min`/`max`, `value`, `lines`,
    /// `validate`, `primary`, `destructive`, `columns`/`rows`, ...
    #[serde(flatten)]
    pub state: Map<String, Value>,
}

fn yes() -> bool {
    true
}

impl Element {
    pub fn new(r#ref: &str, r#type: &str) -> Element {
        Element {
            r#ref: r#ref.to_string(),
            r#type: r#type.to_string(),
            name: None,
            class: Vec::new(),
            help: None,
            required: false,
            default: None,
            secret: false,
            enabled: true,
            error: None,
            state: Map::new(),
        }
    }

    pub fn is_action(&self) -> bool {
        self.r#type == types::ACTION
    }

    /// Whether answering via this action triggers required-field
    /// validation. Only meaningful on actions; defaults to true.
    pub fn validates(&self) -> bool {
        self.state
            .get("validate")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }
}
