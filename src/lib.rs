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

pub mod daemon;
pub mod element;
pub mod frame;
pub mod msg;
pub mod surface;

pub use element::Element;
pub use frame::MsgType;
