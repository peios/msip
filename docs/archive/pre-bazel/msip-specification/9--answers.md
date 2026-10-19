---
title: Answers
description: The one message a bound surface sends — its shape, the action rules, validation, and the three fates an answer can meet.
---

## Shape

A surface answers the outstanding turn with `Answer`:

| Field | Meaning |
|---|---|
| `seq` | The turn being answered. |
| `action` | The action pressed, by ref. At most one. |
| `values` | Input values, keyed by ref. |

`values` MUST name only enabled input elements of the turn named by
`seq`; a value for an unknown ref, a disabled element, an output
element or an action is a protocol error. A value's JSON type must
match the element type's answer encoding (§3.B).

## Actions

If the turn has any enabled action, the answer MUST name exactly one
of them; if it has none, the answer MUST omit `action` — the implicit
submit. Naming anything that is not an enabled action of the turn is
a protocol error.

What an action means — next, back, cancel, begin — is the daemon's
business alone. A surface presents actions and reports which was
pressed; it MUST NOT ascribe navigation semantics of its own.

## Validation

An answer validates unless the action it names carries
`validate: false` (§3.B) — the escape that lets *Back* and *Cancel*
leave a half-filled page. When an answer validates:

- every enabled, required input element MUST have a value in
  `values`;
- an omitted optional input with a `default` is treated as answering
  with that default.

The daemon then applies its own domain validation — is that disk
still present, is that name well-formed — which this chapter does
not constrain.

## The three fates

An answer is exactly one of:

- **Stale** — `seq` is not the outstanding turn. Ignored silently
  (§3.3).
- **Applied** — structurally sound and valid. The acknowledgement is
  whatever the daemon broadcasts next: the following turn, or END.
  The winning answer itself is never echoed to anyone.
- **Failed validation** — structurally sound, but required values are
  missing or domain validation failed. The daemon broadcasts an
  UPDATE setting `error` on the offending elements; the turn stays
  open for every surface (§3.3).

A structurally unsound answer — the protocol errors above — is none
of these: it is answered by `Error` on the offending connection only
(§3.11), and the turn is unaffected.
