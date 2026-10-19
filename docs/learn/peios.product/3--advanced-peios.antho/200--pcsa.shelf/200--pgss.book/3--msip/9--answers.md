---
title: Answers
description: Answer encoding, action selection, required values, and
  arbitration.
---

A bound surface answers the outstanding turn with Answer. The daemon checks
the answer before applying domain-specific rules.

## Encoding

Answer has these fields:

| Field | Meaning |
| --- | --- |
| `seq` | Turn being answered |
| `action` | Selected action ref, if present |
| `values` | Object mapping input refs to values |

The values MUST name only enabled inputs of the turn identified by `seq`.
Unknown refs and values for disabled elements, outputs, or actions are
protocol errors. Each value must match its element's answer encoding (§3.B),
including offered choices and selectable table rows.

## Actions

When a turn has enabled actions, the answer MUST name exactly one. When it has
none, the answer MUST omit `action`. Naming any other element as an action is
a protocol error.

Only the daemon interprets actions such as next, back, or cancel. A surface
MUST NOT infer navigation semantics from action names.

## Validation

An answer validates unless its selected action has `validate: false`. A
nonvalidating action can leave an incomplete page, but supplied values still
obey the structural rules.

For a validating answer:

- Every enabled, required input MUST have an explicit value in `values`.
- An omitted optional input with a default is treated as having that default.

The daemon then performs domain validation, such as checking that a selected
disk still exists. The protocol does not define those application rules.

## Results

An answer has one of these results:

- **Stale:** Its sequence is not outstanding. The daemon ignores it silently.
- **Applied:** It is structurally sound and passes validation. The daemon
  acknowledges it through the next Turn or End, without echoing the answer.
- **Failed validation:** Required input is missing or domain validation fails.
  The daemon broadcasts element errors in an Update and keeps the turn open.

A structural violation instead receives Error on the offending connection. It
does not change the outstanding turn (§3.11).
