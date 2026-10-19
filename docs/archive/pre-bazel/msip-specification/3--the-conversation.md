---
title: The Conversation
description: Who opens, who drives, broadcast to every attached surface, first-valid-answer arbitration, and the lifecycle of a persistent conversation.
---

## A surface opens; the daemon drives

Every conversation is opened by a surface (§3.6). Within it, the
daemon speaks first and always: it issues every turn, and a surface
never sends anything but an answer to the turn that is outstanding.

The invariant is about who *opens*, not who has the need. A daemon
that wants to put a question to whoever is around — a passphrase
prompt at boot, an authorisation request — is served by a surface
that opened an agent-style conversation earlier and sat waiting in
it. The daemon still drives; the surface still merely opened.

## Broadcast, and the first valid answer

A conversation is not a session with one surface. Every attached
connection receives every TURN and every UPDATE, and any of them may
answer the outstanding turn.

Arbitration is by seq:

- The first **valid** answer to the outstanding turn is applied. The
  daemon then either issues the next turn — which is the only
  acknowledgement of acceptance — or ends the conversation.
- An answer that fails validation does not close the turn. The daemon
  broadcasts the failure as element errors (§3.10) and the turn
  remains open to every surface, including the one that failed.
- An answer to any other seq is **stale**, and MUST be ignored
  silently. No reply is owed: the broadcast that made it stale is
  already on the wire to its sender, and reaching the loser is the
  transport's job, not arbitration's.

Two people on two surfaces can race each other's intent. That is
accepted: access control (§3.4) already admitted both, and the
situation is no different from two administrators sharing a shell.

## Persistence

A conversation outlives its connections. A surface that crashes, a
terminal that disconnects, a web page that is closed — none of these
end the conversation; the daemon carries on, and a new connection may
attach and continue where the person left off.

A surface that attaches mid-conversation receives the outstanding
turn in its **current** state — the turn as issued, with every update
since folded in (§3.10). It does not receive history. There is
deliberately no transcript-replay mechanism in this protocol.

## Lifecycle

A conversation exists from the moment the daemon creates it (§3.6)
until the daemon broadcasts END (§3.11), after which its identifier
is meaningless and MUST NOT appear in a listing.

A daemon SHOULD bound the life of an abandoned conversation — one
with no attached surfaces and no progress — by policy of its own:
expiry after an interval, a cancel decision, or indefinite patience
are all legitimate. This chapter requires only that whatever happens
is reported as an END to anyone attached when it happens.

How many conversations of a kind may exist at once is daemon policy,
not protocol. A daemon enforcing a singleton — one ongoing
installation, say — refuses nothing: it answers a START for a kind
that is already running by attaching the caller to the existing
conversation, and says so (§3.6).
