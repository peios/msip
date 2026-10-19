---
title: Ending and Errors
description: END as the conversation's one terminal message, its outcomes, and the strictly-protocol meaning of ERROR.
---

## End

A conversation ends exactly one way: the daemon broadcasts `End`.

| Field | Meaning |
|---|---|
| `seq` | The conversation's final seq. |
| `outcome` | `complete`, `cancelled`, or `failed`. |
| `message` | Optional text for the person. |

`complete` is the flow's own success. `cancelled` reports a decision
— a person pressed the action the daemon interprets as abandonment,
or daemon policy expired an abandoned conversation (§3.3). `failed`
reports that the work cannot continue. In every case the daemon
SHOULD have said more in the preceding turns; `message` is a
last line, not a report.

After END the conversation is over: its identifier MUST NOT appear
in a listing, an `Attach` naming it is refused `unknown_kind`, and
the daemon SHOULD close the connections that were attached. A
surface receiving END renders the outcome and is done.

## Error

`Error` reports a protocol violation on one connection, and nothing
else — it is never a flow outcome, never validation, never staleness.

| `code` | Violation |
|---|---|
| `malformed` | Undecodable body, or a field breaking this chapter's rules. |
| `unbound` | A message requiring a binding, on an unbound connection — or a second binding attempt on a bound one. |
| `unknown_ref` | An answer naming a ref the turn does not have. |
| `bad_value` | A value on an action/disabled/output element, a wrongly-typed value, or an action rule of §3.9 broken. |
| `unexpected_type` | A message type this peer never receives, or one invalid in the connection's state. |

An `Error` is advisory: the sender MAY close the connection after
it, and MUST close it when the violation makes further decoding
unsafe. Errors are per-connection; the conversation, and every other
surface, are unaffected.
