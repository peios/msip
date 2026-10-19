//! The surface side: track the conversation as the daemon drives it.
//!
//! [`Session`] consumes decoded messages and keeps the current
//! [`Page`] — the outstanding turn with every update folded in — so a
//! renderer only ever draws current state.

use std::fmt;

use serde_json::{Map, Value};

use crate::element::Element;
use crate::frame::MsgType;
use crate::patch::{self, PatchError};
use crate::msg::{Answer, Bound, End, ErrorMsg, Listing, Refused, Turn, Update, Welcome};

/// The outstanding turn after all accepted updates have been folded into it.
#[derive(Debug, Clone)]
pub struct Page {
    pub turn: Turn,
}

impl Page {
    /// Look up an element by its stable ref.
    pub fn element(&self, r: &str) -> Option<&Element> {
        self.turn.elements.iter().find(|e| e.r#ref == r)
    }

    fn apply(&mut self, upd: &Update) -> Result<(), SessionError> {
        if upd.seq != self.turn.seq {
            // An update for a turn we do not hold; the TURN broadcast
            // that supersedes it is on the wire. Ignore.
            return Ok(());
        }
        if upd.full {
            let mut elements = Vec::with_capacity(upd.elements.len());
            for m in &upd.elements {
                let e = serde_json::from_value(Value::Object(m.clone()))
                    .map_err(|e| SessionError::Malformed(e.to_string()))?;
                elements.push(e);
            }
            self.turn.elements = elements;
            return Ok(());
        }
        patch::apply(&mut self.turn.elements, &upd.elements).map_err(|error| match error {
            PatchError::UnknownRef(name) => SessionError::UnknownRef(name),
            PatchError::Malformed(message) => SessionError::Malformed(message),
        })?;
        Ok(())
    }
}

/// What a received message meant; the renderer redraws on
/// `NewTurn`/`Updated` and reacts to the rest.
#[derive(Debug)]
pub enum Event {
    Welcome(Welcome),
    Listing(Listing),
    Bound(Bound),
    Refused(Refused),
    NewTurn,
    Updated,
    Ended(End),
    /// The daemon reports a protocol violation by this surface.
    ProtocolError(ErrorMsg),
}

/// A rejected daemon message. Rejected updates and End messages preserve the page.
#[derive(Debug)]
pub enum SessionError {
    Malformed(String),
    UnknownRef(String),
    /// A message type a surface never receives.
    UnexpectedType(MsgType),
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionError::Malformed(e) => write!(f, "malformed body: {e}"),
            SessionError::UnknownRef(r) => write!(f, "update names unknown element {r:?}"),
            SessionError::UnexpectedType(t) => write!(f, "unexpected message type {t:?}"),
        }
    }
}

impl std::error::Error for SessionError {}

/// State for rendering a single conversation.
///
/// The caller enforces greeting, binding, capability negotiation, and transport
/// deadlines. This helper accepts decoded messages without a connection-state gate.
#[derive(Default)]
pub struct Session {
    conversation: Option<String>,
    page: Option<Page>,
}

impl Session {
    /// Create a session without a binding or page.
    pub fn new() -> Session {
        Session::default()
    }

    /// Return the last Bound identifier, retained after End for result display.
    pub fn conversation(&self) -> Option<&str> {
        self.conversation.as_deref()
    }

    /// The outstanding page in its current state.
    pub fn page(&self) -> Option<&Page> {
        self.page.as_ref()
    }

    /// Consume one decoded daemon message.
    pub fn handle(&mut self, t: MsgType, body: Value) -> Result<Event, SessionError> {
        fn parse<T: serde::de::DeserializeOwned>(body: Value) -> Result<T, SessionError> {
            serde_json::from_value(body).map_err(|e| SessionError::Malformed(e.to_string()))
        }
        match t {
            MsgType::Welcome => Ok(Event::Welcome(parse(body)?)),
            MsgType::Listing => Ok(Event::Listing(parse(body)?)),
            MsgType::Bound => {
                let b: Bound = parse(body)?;
                self.conversation = Some(b.conversation.clone());
                Ok(Event::Bound(b))
            }
            MsgType::Refused => Ok(Event::Refused(parse(body)?)),
            MsgType::Turn => {
                let turn: Turn = parse(body)?;
                self.page = Some(Page { turn });
                Ok(Event::NewTurn)
            }
            MsgType::Update => {
                let upd: Update = parse(body)?;
                if let Some(page) = &mut self.page {
                    page.apply(&upd)?;
                }
                Ok(Event::Updated)
            }
            MsgType::End => {
                let end = parse(body)?;
                self.page = None;
                Ok(Event::Ended(end))
            }
            MsgType::Error => Ok(Event::ProtocolError(parse(body)?)),
            other => Err(SessionError::UnexpectedType(other)),
        }
    }

    /// Build an answer to the outstanding page.
    pub fn answer(&self, action: Option<&str>, values: Map<String, Value>) -> Option<Answer> {
        self.page.as_ref().map(|p| Answer {
            seq: p.turn.seq,
            action: action.map(str::to_string),
            values,
        })
    }
}
