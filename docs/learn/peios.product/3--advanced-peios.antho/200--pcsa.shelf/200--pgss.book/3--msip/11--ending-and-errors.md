---
title: Ending and Errors
description: Terminal outcomes and per-connection protocol errors.
---

End finishes a conversation. Error reports a protocol violation on one
connection and does not finish the operation.

## End

The daemon broadcasts End with these fields:

| Field | Meaning |
| --- | --- |
| `seq` | Final sequence number |
| `outcome` | `complete`, `cancelled`, or `failed` |
| `message` | Optional explanatory text |

`complete` reports success. `cancelled` reports abandonment by an action or
daemon policy. `failed` reports that the operation cannot continue. The daemon
SHOULD explain the result in preceding turns; the final message is a short
conclusion.

After END, the identifier MUST NOT appear in listings. Attach is refused with
`unknown_kind`. The daemon SHOULD close attached connections. A surface
renders the outcome and finishes the conversation.

## Error

Error applies only to protocol violations. It does not represent staleness,
validation failure, or an operation result. Its codes are:

| Code | Violation |
| --- | --- |
| `malformed` | Invalid body or field |
| `unbound` | A message requires a binding, or a bound connection attempts another binding |
| `unknown_ref` | An answer names an absent ref |
| `bad_value` | Invalid input encoding, an ineligible element value, or an action-rule violation |
| `unexpected_type` | A message is invalid for the peer or connection state |

The sender MAY close the connection after Error. It MUST close it when further
decoding is unsafe. Other connections and the conversation remain unaffected.
