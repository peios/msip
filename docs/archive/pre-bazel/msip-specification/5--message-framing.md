---
title: Message Framing
description: The 12-byte header MSIP shares with the PGSS codec, its JSON body, and the extensibility rules for each.
---

## Header

MSIP reuses the header layout of §2.6 with its own magic. Every
message begins with a 12-byte header:

| Offset | Size | Field | Value |
|---|---|---|---|
| 0 | 4 | `magic` | `MSIP` (`4d 53 49 50`) |
| 4 | 2 | `version` | `1` |
| 6 | 2 | `msg_type` | See §3.A |
| 8 | 4 | `total_len` | Header plus body, in bytes |

The rules of §2.6 apply unchanged to the magic (checked on every
message; wrong magic is an immediate whole-connection failure), to
the version (an unimplemented version MUST be refused; here, by
closing the connection, optionally after an `Error` where one can
still be encoded), and to `total_len` (self-delimiting; a decoder
MUST reject a declaration exceeding the limit without reading the
body, and one smaller than the header).

The high bit of `msg_type` marks a message sent by the **daemon**;
surface messages have it clear. As in §2.6, this is a readability
property — a peer MUST validate the type it received against what it
expected, not merely against the direction bit.

A message MUST NOT exceed **1 MiB** in total. The limit is larger
than the logon channel's because `log` elements legitimately carry
bulk; it is a limit all the same.

## Body

The body is one JSON object, UTF-8 encoded, as defined by RFC 8259.
Which object each message type carries is specified in §3.6 to
§3.11 and tabulated in §3.A. The message type lives in the header
only; the body does not repeat it.

A decoder MUST reject a body that is not valid UTF-8, not valid
JSON, or not an object, as a protocol error (§3.11).

## Extensibility

The JSON body extends by different rules than the binary codec, and
they are binding on anyone revising this chapter:

- A decoder MUST ignore an object key it does not know. New fields
  are added as new keys, and MUST be optional with a safe default,
  because older peers will not send them and will not read them.
- An absent optional field and a field carrying its documented
  default MUST NOT be distinguished.
- A peer MUST treat an unknown value of an enumerated field —
  `Refused.reason`, `Error.code`, `End.outcome` — as a protocol
  error. Adding a value to an enumeration is a breaking change and
  requires a version bump.

Element **types** and element **state** are the deliberate exception,
as `supported_credential_types` is to Logon (§2.6): the set a surface
renders is a statement of capability negotiated at HELLO, a daemon
MUST NOT send an element type the surface did not declare, and a
type's state keys are defined by the element type (§3.B), not by the
message format. Adding an element type is therefore not a version
bump — an old surface simply does not declare it.
