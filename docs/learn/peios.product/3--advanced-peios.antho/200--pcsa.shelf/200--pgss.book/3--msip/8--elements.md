---
title: Elements
description: Common element fields, semantic references, and input roles.
---

An element is a JSON object with common fields and type-specific state. Its
ref identifies the meaning of the element rather than its screen position.

## Common fields

The common fields and defaults are:

| Field | Default | Meaning |
| --- | --- | --- |
| `ref` | Required | Semantic identity, unique within the turn (§3.2) |
| `type` | Required | Negotiated element type |
| `name` | Absent | Display label |
| `class` | `[]` | Presentation hints |
| `help` | Absent | Explanatory text |
| `required` | `false` | A validating answer must supply a value |
| `default` | Absent | Value assumed when an optional input is omitted |
| `secret` | `false` | Secret-handling rules apply (§3.4) |
| `enabled` | `true` | Whether the element can be answered |
| `error` | Absent | Validation error set by UPDATE |

Additional keys are type-specific state (§3.B). A surface MAY substitute
translations keyed by element ref or turn id, with the supplied `name` and
`help` as fallbacks.

## Unavailable elements

An element with `enabled: false` remains on the page. A surface SHOULD show it
as unavailable, with `help` explaining why. It MUST NOT collect input for it.
An answer supplying a value for it is a protocol error.

## Roles

Element types have three roles:

- **Outputs:** `text`, `progress`, and `log` provide information. They receive
  no answers and are exempt from `required`.
- **Inputs:** `string`, `boolean`, `select`, and `table` collect typed values
  under their refs.
- **Actions:** `action` reports a selection without a value. An automated
  surface that can answer all inputs can select the primary action without
  interpreting navigation semantics.
