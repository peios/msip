---
title: The MSIP Rust Library
type: concept
description: Library responsibilities, caller-owned protocol duties, and
  integration boundaries for MSIP daemons and surfaces.
---

The multi-surface interaction protocol (MSIP) lets one daemon drive terminal,
graphical, and automated interfaces. The `msip` Rust crate provides wire
framing, message structures, answer checks, and page state. It doesn't open
listeners or run an operation for you.

Use the [protocol specification](~specifications/pgss/msip/scope-and-roles) for the
complete contract. The library is a set of helpers, not a conformance claim
for an application that embeds it.

## Library boundaries

The modules divide responsibilities as follows:

| Module | Responsibility |
| --- | --- |
| `frame` | Version-one framing, a 1 MiB total limit, and JSON object bodies |
| `msg` | Typed message bodies and protocol enum encodings |
| `element` | Common element fields and open type-specific state |
| `daemon` | Turn sequencing, atomic updates, and structural answer checks |
| `surface` | The outstanding page after accepted updates |

Your application owns socket permissions, peer verification, authorization,
HELLO negotiation, binding order, listings, and conversation lifetime. It
also owns timeouts, write serialization, bounded queues, and broadcast to
attached peers. A frame's direction bit doesn't establish a valid connection
state.

## Daemon state

Create a `Conversation` for an application-defined kind. Creation reads
128 random bits from `/dev/urandom`; failure returns an I/O error. The
identifier names the conversation and does not authorize attachment.

Supply a `TurnSpec` to `issue`. It assigns the next sequence number and
replaces the outstanding turn. Provide unique refs, valid state and defaults,
and only element types supported by every attached surface.

Call `handle_answer` for a decoded Answer. It returns stale, protocol-error,
missing-required-value, or valid results. It checks built-in value encodings
and select/table choices. Your application validates domain rules, including
string length, patterns, and whether referenced resources still exist.
Negotiated extension types also need application-specific checks.

A valid result doesn't consume the answer. Serialize checking, domain
validation, and acceptance across attached surfaces. Accept by issuing the
next turn or ending the conversation before processing another competing
answer. Reject domain validation with `reject`, then broadcast its Update.

## Updates and attachment

`update` stages an entire sparse batch before changing the stored turn.
Unknown refs or malformed fields reject the batch without partial changes.
Repeated refs fold in order, and log `append` entries accumulate in `lines`.
If a log patch also supplies `lines`, that explicit value replaces the
accumulated lines. Prefer one operation per patch.

`update_full` replaces the element list. Neither operation changes the turn
sequence. Send `current_turn` after a successful attachment so the surface
receives folded state without replaying earlier updates.

`end` marks the conversation ended and returns End for broadcast. The
embedding daemon removes it from listings and handles connection cleanup.
The library retains its final turn until the conversation is dropped.

## Surface state

Pass decoded daemon messages to `Session::handle`. Turn replaces the page;
Update changes it atomically. Updates for other sequence numbers are ignored.
Malformed updates and End bodies preserve the page. A valid End clears it.
The last Bound identifier remains available for displaying the result.

The session doesn't enforce greeting or binding order. Apply those checks in
your connection handler before calling it. Treat names, help, and messages
as text; don't execute instructions inferred from display content.

## Secrets and failure handling

Mark secret inputs, collect them without echo, and avoid retaining their
values after submission. Never put collected secrets in turn state, updates,
errors, or logs. Message and element `Debug` implementations do not redact
values. The library does not wipe caller-owned buffers.

A framing error can leave the stream between messages. Close the connection
instead of attempting to resume decoding. A failed write can have sent a
partial frame; don't retry that frame on the same stream. Configure read and
write deadlines in the transport layer.

## Presentation across surfaces

Turn and element classes are optional presentation hints. The protocol
carries typed content and state; it does not prescribe a daemon-controlled
layout. Keep actions, required values, and validation correct when a surface
ignores every class.

The installer owns its terminal renderer's
[presentation behavior](~peios/setup-conversations/terminal-presentation).
Those conventions describe `msip-tui`, not guarantees made by the Rust
library or by all MSIP surfaces.
