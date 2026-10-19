---
title: The Conversation
description: Conversation ownership, broadcast arbitration, persistence, and
  lifetime.
---

A surface opens a conversation, and the daemon drives it. The daemon issues
every turn; a bound surface answers the outstanding turn.

## Opening a conversation

A daemon that needs to request input uses a conversation opened by a surface.
For example, a surface can open a persistent conversation and wait for a
passphrase request. The daemon still chooses when to issue a turn.

## Broadcasting and arbitration

Every attached connection receives every TURN and UPDATE. Any attached surface
can answer the outstanding turn. The daemon arbitrates answers by `seq`:

- Apply the first valid answer, then issue the next turn or end the
  conversation. That broadcast acknowledges acceptance.
- Keep the turn open when validation fails. Broadcast element errors so every
  surface can correct its answer (§3.10).
- Ignore an answer for any other sequence number silently. A stale answer MUST
  NOT receive a protocol error.

Two authorized surfaces can submit competing answers. The protocol does not
designate a controlling surface or provide takeover operations.

## Persistence

Disconnecting, closing, or crashing a surface does not end the conversation.
The daemon continues the operation. A replacement connection can attach and
receive the outstanding turn with all updates folded into it (§3.10).

Attachment provides current state, not a transcript. The protocol has no
history-replay operation.

## Lifetime and multiplicity

A conversation exists from creation until END (§3.11). Its identifier MUST NOT
appear in listings after it ends.

A daemon SHOULD define a policy for abandoned conversations with no attached
surfaces and no progress. It can expire, cancel, or retain them. If a
conversation ends, the daemon reports END to every connection still attached.

The daemon defines how many conversations of each kind can exist. For a
singleton kind, a START while one exists attaches the caller to it and reports
`attached: true` (§3.6).
