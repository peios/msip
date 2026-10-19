---
title: Message Framing
description: The version-one header, JSON object body, limits, and extension
  rules.
---

Every multi-surface interaction protocol (MSIP) message has a 12-byte header
followed by one UTF-8 JSON object. The header uses the layout from §2.6 and
its own protocol magic.

## Header

Integer fields are little-endian. The header fields are:

| Offset | Bytes | Field | Value |
| --- | --- | --- | --- |
| 0 | 4 | `magic` | `MSIP` (`4D 53 49 50`) |
| 4 | 2 | `version` | `1` |
| 6 | 2 | `msg_type` | Message identifier (§3.A) |
| 8 | 4 | `total_len` | Header and body length in bytes |

A decoder MUST check the magic on every message. Wrong magic is an immediate
connection failure. It MUST refuse an unsupported version by closing the
connection, optionally sending an Error if encoding remains safe.

A message MUST NOT exceed 1 MiB including its header. A decoder MUST reject a
declared length above this limit without reading the body. It MUST also reject
a length smaller than the header.

The high bit of `msg_type` is set for daemon messages and clear for surface
messages. A peer MUST check the actual type against its expected connection
state; checking direction alone is insufficient.

## Body

The body is one JSON object as defined by RFC 8259. The header carries the
message type; the body does not repeat it. Message fields are specified in
§3.6 to §3.11 and summarized in §3.A.

A decoder MUST reject invalid UTF-8, invalid JSON, and JSON values that are
not objects as protocol errors (§3.11).

## Extensibility

Extensions follow these rules:

- A decoder MUST ignore unknown object keys. Additional fields MUST be
  optional and have safe defaults for older peers.
- An absent optional field and its documented default MUST NOT be
  distinguished.
- Unknown values of `Refused.reason`, `Error.code`, and `End.outcome` MUST
  produce protocol errors. Extending those enumerations requires a protocol
  version change.

Element types use capability negotiation instead of a version change. A daemon
MUST NOT send a type absent from HELLO. Each type defines its own state keys
(§3.B). A surface that does not implement an added type omits it from HELLO.
