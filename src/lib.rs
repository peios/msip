//! MSIP — the Multi-Surface Interaction Protocol (PGSS MSIP).
//!
//! A conversational protocol by which a daemon (the *server* of a
//! conversation) drives one or more generic UI *surfaces*: it issues
//! turns — pages of typed elements — and surfaces render them and
//! answer. Any number of surfaces may attach to one conversation;
//! every turn is broadcast to all of them and the first valid answer
//! wins.
//!
//! This crate is transport-agnostic above the framing layer: it
//! provides the wire codec ([`frame`]), the message bodies ([`msg`]),
//! the element model ([`element`]), and the two state machines — the
//! daemon side's [`daemon::Conversation`] and the surface side's
//! [`surface::Session`]. Socket handling, broadcast fan-out and access
//! control belong to the embedding daemon.

//! Issue a turn and interpret a structurally valid answer.
//!
//! ```
//! use msip::daemon::{Conversation, Disposition, TurnSpec};
//! use msip::element::{Element, types};
//! use msip::msg::Answer;
//! use serde_json::json;
//!
//! let mut conversation = Conversation::new("settings")?;
//! let mut name = Element::new("name", types::STRING);
//! name.required = true;
//! conversation.issue(TurnSpec {
//!     elements: vec![name],
//!     ..Default::default()
//! })?;
//! let answer = Answer {
//!     seq: conversation.seq(),
//!     action: None,
//!     values: [("name".into(), json!("workstation"))].into_iter().collect(),
//! };
//! assert!(matches!(conversation.handle_answer(&answer), Disposition::Valid(_)));
//! // Apply domain validation, then issue the next turn or end the conversation
//! // before accepting another answer. This check alone does not consume it.
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod daemon;
pub mod element;
pub mod frame;
pub mod msg;
mod patch;
pub mod surface;

pub use element::Element;
pub use frame::MsgType;
