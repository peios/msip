---
title: Binding
description: Greeting, authorization, and binding one connection to one
  conversation.
---

A connection carries at most one conversation during its lifetime. Closing the
connection detaches the surface. A surface needing two conversations opens two
connections.

## Greeting

The first message MUST be the surface's Hello. Its fields are:

| Field | Meaning |
| --- | --- |
| `element_types` | Types that the surface can render |
| `surface` | Optional observability string, such as `install-tui/0.1` |

The daemon replies with Welcome, listing its conversation `kinds` and an
optional daemon observability string. The daemon MUST NOT vary behavior based
on `surface`.

The daemon MUST NOT send an element type absent from `element_types`. It MUST
refuse a binding with `unsupported_elements` when the conversation requires
types the surface cannot render.

## Unbound connections

After WELCOME, a connection can send these messages:

- **List.** Receive a Listing of conversations the caller can attach to. An
  empty result is valid.
- **Start.** Open a conversation of a specified kind.
- **Attach.** Join a conversation by identifier.

Other messages on an unbound connection are protocol errors. A second binding
attempt on a bound connection is also a protocol error.

## Successful binding

A successful Start or Attach receives Bound with the conversation identifier,
current `seq`, and `attached`. The latter is true when joining an existing
conversation. If a turn is outstanding, the daemon MUST immediately send its
folded state after Bound.

A daemon MAY implement a singleton kind by answering Start with an attachment
to its existing conversation. The reply reports `attached: true`.

A conversation identifier is 32 lowercase hexadecimal characters encoding 128
bits from a cryptographically secure source. It appears in Attach, Bound, and
Listing.

## Refusal

Refused leaves the connection unbound. Its reasons are:

| Reason | Meaning |
| --- | --- |
| `access_denied` | Authorization failed |
| `unknown_kind` | The kind is unknown, or the conversation is absent or ended |
| `unsupported_elements` | The conversation requires undeclared element types |
| `unavailable` | The daemon offers the kind but cannot open a conversation |

The daemon MUST use Refused only for binding attempts. It SHOULD prefer
`unknown_kind` when the caller must not learn that a conversation exists. For
`unavailable`, it SHOULD explain the failure in `message` without disclosing
secrets.
