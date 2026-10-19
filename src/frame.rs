//! Wire framing: the PGSS §2.6 header shape with a JSON body.
//!
//! Every message is a 12-byte header — magic, version, message type,
//! total length — followed by a JSON object. A wrong magic is a
//! whole-connection failure: Peios protocols share this header layout,
//! and a socket plugged into the wrong daemon must fail on byte zero,
//! not several fields in.

use std::fmt;
use std::io::{self, Read, Write};

use serde::Serialize;

/// Four-byte discriminator checked on every frame.
pub const MAGIC: [u8; 4] = *b"MSIP";
/// Supported MSIP wire version.
pub const VERSION: u16 = 1;
/// Wire header size in bytes.
pub const HEADER_LEN: usize = 12;
/// Header plus body. Larger than Logon's 64 KiB because `log` elements
/// legitimately carry bulk.
pub const MAX_TOTAL_LEN: u32 = 1 << 20;
/// Set on every message sent by the daemon; clear on surface messages.
pub const DAEMON_BIT: u16 = 0x8000;

/// Version-one message identifiers; direction is encoded in the high bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MsgType {
    // surface -> daemon
    Hello = 0x0001,
    List = 0x0002,
    Start = 0x0003,
    Attach = 0x0004,
    Answer = 0x0005,
    // daemon -> surface
    Welcome = DAEMON_BIT | 0x0001,
    Listing = DAEMON_BIT | 0x0002,
    Bound = DAEMON_BIT | 0x0003,
    Refused = DAEMON_BIT | 0x0004,
    Turn = DAEMON_BIT | 0x0005,
    Update = DAEMON_BIT | 0x0006,
    Error = DAEMON_BIT | 0x0007,
    End = DAEMON_BIT | 0x0008,
}

impl MsgType {
    /// Whether this message type is sent by a daemon.
    pub fn from_daemon(self) -> bool {
        (self as u16) & DAEMON_BIT != 0
    }

    fn from_u16(v: u16) -> Option<MsgType> {
        use MsgType::*;
        Some(match v {
            0x0001 => Hello,
            0x0002 => List,
            0x0003 => Start,
            0x0004 => Attach,
            0x0005 => Answer,
            v if v == Welcome as u16 => Welcome,
            v if v == Listing as u16 => Listing,
            v if v == Bound as u16 => Bound,
            v if v == Refused as u16 => Refused,
            v if v == Turn as u16 => Turn,
            v if v == Update as u16 => Update,
            v if v == Error as u16 => Error,
            v if v == End as u16 => End,
            _ => return None,
        })
    }
}

/// Invalid framing, JSON, or an underlying stream error.
#[derive(Debug)]
pub enum FrameError {
    Io(io::Error),
    BadMagic([u8; 4]),
    UnsupportedVersion(u16),
    UnknownType(u16),
    /// total_len exceeds MAX_TOTAL_LEN.
    TooLarge(u32),
    /// total_len smaller than the header itself.
    TooSmall(u32),
    Json(serde_json::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Io(e) => write!(f, "i/o: {e}"),
            FrameError::BadMagic(m) => write!(f, "bad magic {m:02x?}"),
            FrameError::UnsupportedVersion(v) => write!(f, "unsupported version {v}"),
            FrameError::UnknownType(t) => write!(f, "unknown message type {t:#06x}"),
            FrameError::TooLarge(n) => write!(f, "declared length {n} exceeds limit"),
            FrameError::TooSmall(n) => write!(f, "declared length {n} smaller than header"),
            FrameError::Json(e) => write!(f, "body: {e}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for FrameError {
    fn from(e: io::Error) -> Self {
        FrameError::Io(e)
    }
}

/// Encode one JSON object, write its frame, and flush the stream.
///
/// Invalid bodies and oversized frames fail before any bytes are written. An
/// I/O error can leave a partial frame; the caller must abandon that connection.
/// Serialize concurrent writes and configure transport deadlines in the caller.
pub fn write_msg<W: Write, T: Serialize>(
    writer: &mut W,
    message_type: MsgType,
    body: &T,
) -> Result<(), FrameError> {
    let body = serde_json::to_vec(body).map_err(FrameError::Json)?;
    // serde_json::to_vec emits valid JSON without leading whitespace. Checking
    // its first byte preserves the serialized field order and avoids reparsing.
    if body.first() != Some(&b'{') {
        return Err(FrameError::Json(serde::ser::Error::custom(
            "MSIP body must be an object",
        )));
    }
    let total = HEADER_LEN.saturating_add(body.len());
    if total > MAX_TOTAL_LEN as usize {
        return Err(FrameError::TooLarge(
            u32::try_from(total).unwrap_or(u32::MAX),
        ));
    }
    let mut header = [0u8; HEADER_LEN];
    header[0..4].copy_from_slice(&MAGIC);
    header[4..6].copy_from_slice(&VERSION.to_le_bytes());
    header[6..8].copy_from_slice(&(message_type as u16).to_le_bytes());
    header[8..12].copy_from_slice(&(total as u32).to_le_bytes());
    writer.write_all(&header)?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

/// Read one bounded frame containing a JSON object.
///
/// Header validation precedes body allocation. Any failure can leave the stream
/// between messages; close it instead of attempting to resume decoding. The caller
/// supplies read deadlines and checks message direction, schema, and connection state.
pub fn read_msg<R: Read>(reader: &mut R) -> Result<(MsgType, serde_json::Value), FrameError> {
    let mut header = [0u8; HEADER_LEN];
    reader.read_exact(&mut header)?;
    if header[0..4] != MAGIC {
        return Err(FrameError::BadMagic([
            header[0], header[1], header[2], header[3],
        ]));
    }
    let version = u16::from_le_bytes([header[4], header[5]]);
    if version != VERSION {
        return Err(FrameError::UnsupportedVersion(version));
    }
    let raw_type = u16::from_le_bytes([header[6], header[7]]);
    let message_type = MsgType::from_u16(raw_type).ok_or(FrameError::UnknownType(raw_type))?;
    let total = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
    if total > MAX_TOTAL_LEN {
        return Err(FrameError::TooLarge(total));
    }
    if (total as usize) < HEADER_LEN {
        return Err(FrameError::TooSmall(total));
    }
    let mut body = vec![0u8; total as usize - HEADER_LEN];
    reader.read_exact(&mut body)?;
    let object = serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&body)
        .map_err(FrameError::Json)?;
    let value = serde_json::Value::Object(object);
    Ok((message_type, value))
}
