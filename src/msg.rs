//! Message bodies. The message *type* lives in the frame header
//! (§frame); each body here is the JSON object that follows it.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::element::Element;

/// Surface -> daemon, first message on a connection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hello {
    /// Observability only ("install-peios-tui/0.1"). A daemon MUST NOT
    /// vary behavior on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<String>,
    /// Element types this surface can render. A daemon that needs a
    /// type not listed refuses at bind time (`unsupported_elements`)
    /// rather than sending an unrenderable page.
    pub element_types: Vec<String>,
}

/// Daemon greeting with supported conversation kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Welcome {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daemon: Option<String>,
    pub kinds: Vec<String>,
}

/// Request the conversations this peer is authorized to attach to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct List {}

/// One authorized conversation in a listing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListingEntry {
    pub conversation: String,
    pub kind: String,
    pub seq: u64,
}

/// Conversations visible to the requesting peer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Listing {
    pub conversations: Vec<ListingEntry>,
}

/// Binds the connection. The daemon MAY answer a Start by attaching to
/// an existing conversation (installerd's singleton does exactly that);
/// `Bound::attached` says which happened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Start {
    pub kind: String,
}

/// Request attachment to an existing conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attach {
    pub conversation: String,
}

/// Successful binding to a newly created or existing conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bound {
    pub conversation: String,
    /// Current sequence number; a TURN for it follows immediately when
    /// one is outstanding.
    pub seq: u64,
    /// True when a Start joined an existing conversation.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub attached: bool,
}

/// Binding refusal reasons; unknown values are protocol errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusedReason {
    AccessDenied,
    UnknownKind,
    UnsupportedElements,
    /// The kind exists, but the daemon cannot open a conversation.
    Unavailable,
}

/// A refused binding; the connection remains unbound.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Refused {
    pub reason: RefusedReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// A page. `seq` is the only thing a surface cites; `id` is a template
/// identity (same id => same set of element refs and types) for
/// client-side formatting, and is not unique within a conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Turn {
    pub seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class: Vec<String>,
    pub elements: Vec<Element>,
}

/// Element state pushed within the current turn. Sparse patches touch
/// existing refs only; `full: true` replaces the element list and is
/// required whenever the set of refs changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Update {
    pub seq: u64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub full: bool,
    /// Sparse: objects carrying `ref` plus the changed fields.
    /// Full: complete elements.
    pub elements: Vec<Map<String, Value>>,
}

/// A surface's answer to the outstanding turn. At most one action; a
/// turn with no action elements is answered with `action` absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
    pub seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub values: Map<String, Value>,
}

/// Protocol violation only — never a flow outcome. (A stale answer is
/// silently ignored; a validation failure is an Update carrying
/// element errors.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Malformed,
    Unbound,
    UnknownRef,
    BadValue,
    UnexpectedType,
}

/// A connection-local protocol violation, with optional display text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorMsg {
    pub code: ErrorCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Terminal operation outcomes; the wire spelling `cancelled` is preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Complete,
    Cancelled,
    Failed,
}

/// The terminal message broadcast to every attached surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct End {
    pub seq: u64,
    pub outcome: Outcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
