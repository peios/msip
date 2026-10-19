---
title: Elements
description: The fields every element carries, what each obliges the two roles to do, and the discipline that keeps elements semantic.
---

## Common fields

Every element, of every type, is a JSON object with these fields:

| Field | Default | Meaning |
|---|---|---|
| `ref` | — | Semantic identity (§3.2). Unique within the turn. |
| `type` | — | An element type from the negotiated set. |
| `name` | absent | Display label. |
| `class` | `[]` | Presentation hints. |
| `help` | absent | Longer explanatory text. |
| `required` | `false` | An answer that validates must supply a value. |
| `default` | absent | Value assumed when the answer omits this ref. |
| `secret` | `false` | §3.4's handling rules apply. |
| `enabled` | `true` | Disabled elements render but cannot be answered. |
| `error` | absent | Validation failure text, set by UPDATE (§3.10). |

All further keys are **state**, defined per element type (§3.B).

`name` and `help` are for the person. A surface MAY substitute its
own text — translations keyed on the element's `ref` or the turn's
`id` — and the daemon-supplied text is the fallback; this is the
protocol's entire localisation mechanism, and it is why refs and
template ids are stable.

## Disabled elements

An element with `enabled: false` is present but inert: a surface
SHOULD render it as visibly unavailable (with its `help` explaining
why), MUST NOT collect input for it, and an answer carrying a value
for it is a protocol error. Disabled elements exist so a flow can
show what it will someday ask — a capability the daemon does not yet
have loses none of its place on the page by being unimplemented.

## Inputs, outputs, and actions

Element types divide three ways, and the division is what a surface
may rely on:

- **Output** types (`text`, `progress`, `log`) inform. They are never
  answered, are exempt from `required`, and exist to be updated in
  place.
- **Input** types (`string`, `boolean`, `select`) collect a typed
  value, returned in the answer under the element's ref.
- **`action`** is pressed, not filled. Actions carry no value and no
  unattended meaning beyond flow: an automated surface that can
  answer every input on a page from its own sources presses the
  page's primary action and needs to understand nothing else.
