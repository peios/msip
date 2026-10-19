---
title: Binding
description: HELLO and WELCOME, the unbound connection, and the three ways a connection binds to a conversation — START, START-as-attach, and ATTACH.
---

## One conversation per connection

A connection carries at most one conversation, for its whole life.
There is no multiplexing: a surface that wants two conversations
opens two connections, and closing the connection is how a surface
detaches. What a connection may send depends only on which of three
states it is in: expecting-hello, unbound, or bound.

## Greeting

The first message on a connection MUST be the surface's `Hello`:

| Field | Meaning |
|---|---|
| `element_types` | The element types this surface can render. |
| `surface` | Optional observability string ("install-tui/0.1"). |

`element_types` is a statement of capability. The daemon MUST NOT
send this connection an element whose type is absent from it, and
MUST refuse a binding whose conversation needs more than the surface
declared (`unsupported_elements`) — refusing at bind time is what
makes an unrenderable page unreachable, rather than a failure
mid-flow. `surface` is for logs; the daemon MUST NOT vary behaviour
on it.

The daemon replies `Welcome`, carrying the conversation `kinds` it
offers and its own optional observability string.

## The unbound connection

After WELCOME the connection is unbound. It may send exactly three
things:

- **`List`** — the daemon replies `Listing`: the conversations this
  caller could attach to, each with its identifier, kind, and current
  seq. A daemon with nothing listable replies with an empty listing.
- **`Start`** — open a conversation of a kind.
- **`Attach`** — join the conversation named by identifier.

Anything else on an unbound connection is a protocol error, as is a
second binding attempt on a bound one.

## Binding

A successful `Start` or `Attach` binds the connection and is answered
by `Bound`, carrying the conversation's identifier, its current
`seq`, and `attached` — true when the caller joined a conversation
that already existed. Immediately after a `Bound`, if a turn is
outstanding, the daemon MUST send it to the new connection in its
current state (§3.3).

A daemon MAY answer a `Start` by attaching: a daemon whose policy
admits one conversation of a kind answers a second `Start` with
`Bound { attached: true }` for the existing one. The surface learns
what happened from `attached` and nothing else changes.

A conversation's identifier is assigned by the daemon at creation:
32 lowercase hexadecimal characters encoding 128 bits that MUST come
from a cryptographically secure source. It names the conversation in
`Attach`, `Bound` and `Listing`, and nowhere else.

A binding that is refused is answered by `Refused` and leaves the
connection unbound:

| `reason` | Meaning |
|---|---|
| `access_denied` | The access check of §3.4 failed. |
| `unknown_kind` | `Start` named a kind this daemon does not offer, or `Attach` named a conversation that does not exist or has ended. |
| `unsupported_elements` | The conversation needs element types the surface did not declare. |
| `unavailable` | The daemon offers this kind but could not open a conversation. |

A daemon SHOULD prefer `unknown_kind` over `access_denied` for a
conversation the caller may not know exists, and MUST NOT use
`Refused` for anything but a binding attempt.

`unavailable` is the daemon's own failure and nothing to do with the
surface: the kind exists, the caller may have it, and the surface can
render it. A daemon SHOULD say what went wrong in `message`. Without
this reason such a daemon can only hang up, and a surface cannot tell
a broken daemon from a slow one — it waits on a turn that will never
come, showing nothing.
