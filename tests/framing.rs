//! Frame boundaries and object-only bodies, independent of message schemas.

use std::error::Error;
use std::io;

use serde_json::{Value, json};

use msip::frame::{FrameError, HEADER_LEN, MAX_TOTAL_LEN, MsgType, read_msg, write_msg};

fn frame(body: &[u8]) -> Vec<u8> {
    let mut bytes = b"MSIP\x01\x00\x01\x00".to_vec();
    bytes.extend_from_slice(&((HEADER_LEN + body.len()) as u32).to_le_bytes());
    bytes.extend_from_slice(body);
    bytes
}

#[test]
fn objects_preserve_the_version_one_wire_format() {
    let body = json!({"extension": [null, true, 3, "hello"]});
    let mut bytes = Vec::new();
    write_msg(&mut bytes, MsgType::Hello, &body).unwrap();
    assert_eq!(bytes, frame(br#"{"extension":[null,true,3,"hello"]}"#));
    assert_eq!(
        read_msg(&mut bytes.as_slice()).unwrap(),
        (MsgType::Hello, body)
    );
    assert_eq!(
        read_msg(&mut frame(b" \n { } \t").as_slice()).unwrap().1,
        json!({})
    );
}

#[test]
fn nonobjects_are_rejected_in_both_directions_without_partial_writes() {
    for body in [
        Value::Null,
        json!([]),
        json!([{}]),
        json!("text"),
        json!(true),
        json!(42),
    ] {
        let mut bytes = Vec::new();
        assert!(matches!(
            write_msg(&mut bytes, MsgType::Hello, &body),
            Err(FrameError::Json(_))
        ));
        assert!(bytes.is_empty());
        let wire = frame(&serde_json::to_vec(&body).unwrap());
        assert!(matches!(
            read_msg(&mut wire.as_slice()),
            Err(FrameError::Json(_))
        ));
    }
}

#[test]
fn exact_limit_round_trips_and_larger_bodies_are_rejected() {
    let overhead = HEADER_LEN + br#"{"text":""}"#.len();
    let body = json!({"text": "x".repeat(MAX_TOTAL_LEN as usize - overhead)});
    let mut bytes = Vec::new();
    write_msg(&mut bytes, MsgType::Hello, &body).unwrap();
    assert_eq!(bytes.len(), MAX_TOTAL_LEN as usize);
    assert_eq!(read_msg(&mut bytes.as_slice()).unwrap().1, body);
    let oversized = json!({"text": "x".repeat(MAX_TOTAL_LEN as usize - overhead + 1)});
    bytes.clear();
    assert!(matches!(
        write_msg(&mut bytes, MsgType::Hello, &oversized),
        Err(FrameError::TooLarge(_))
    ));
    assert!(bytes.is_empty());
}

#[test]
fn malformed_and_truncated_frames_fail() {
    for body in [b"{".as_slice(), b"{\xff}", b"{}{}", b""] {
        assert!(matches!(
            read_msg(&mut frame(body).as_slice()),
            Err(FrameError::Json(_))
        ));
    }
    let complete = frame(b"{}");
    for length in 0..complete.len() {
        let error = read_msg(&mut &complete[..length]).unwrap_err();
        assert!(
            matches!(&error, FrameError::Io(error) if error.kind() == io::ErrorKind::UnexpectedEof)
        );
        assert!(error.source().is_some());
    }
    let mut bytes = complete.clone();
    bytes[4..6].copy_from_slice(&2u16.to_le_bytes());
    assert!(matches!(
        read_msg(&mut bytes.as_slice()),
        Err(FrameError::UnsupportedVersion(2))
    ));
    bytes = complete.clone();
    bytes[6..8].copy_from_slice(&0x7fffu16.to_le_bytes());
    assert!(matches!(
        read_msg(&mut bytes.as_slice()),
        Err(FrameError::UnknownType(0x7fff))
    ));
    bytes = complete;
    bytes[8..12].copy_from_slice(&11u32.to_le_bytes());
    assert!(matches!(
        read_msg(&mut bytes.as_slice()),
        Err(FrameError::TooSmall(11))
    ));
}
