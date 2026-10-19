---
title: Element Types
description: The element-type set — state fields and answer encodings for text, string, boolean, select, table, progress, log and action.
---

The initial set. Each type lists its state fields (beyond the common
fields of §3.8) and, for input types, the JSON encoding of its answer
value. The set grows by capability negotiation, not by version bump
(§3.5); a new type is specified by adding it here.

## `text` — output

Prose for the person to read.

| State | Meaning |
|---|---|
| `text` | The content. |
| `detail` | What the content says, as structure. Optional; see *Detail*, at the end of this chapter. |

## `string` — input

One line of text; with `secret`, a password collector.

| State | Meaning |
|---|---|
| `min`, `max` | Length bounds, in characters. |
| `pattern` | Anchored regular expression the value must match. |
| `placeholder` | Ghost text; not a default. |

Answer value: JSON string.

## `boolean` — input

A yes-or-no.

Answer value: JSON true/false.

## `select` — input

Choose from what the daemon offers.

| State | Meaning |
|---|---|
| `choices` | Array of `{value, name, help?}`, in display order. |
| `multiple` | Zero or more choices rather than exactly one. |

Answer value: one choice's `value`; with `multiple`, an array of
them. Values are compared as exact JSON values.

## `table` — input

Choose among rows that have more to say than a name: a `select`
whose choices are records. Disks, network interfaces, existing
installations — anything a person compares on several facts before
picking one.

| State | Meaning |
|---|---|
| `columns` | Array of `{key, name, align?}`, in display order. `align` is `left` (default) or `right`; numbers and sizes read better right-aligned. |
| `rows` | Array of `{value, cells, enabled?, note?, detail?}`, in display order. `cells` maps a column `key` to display text; a key a row omits renders empty. `enabled` defaults to true; a false row is listed but is not an acceptable answer. `note` is a short remark shown with the row — typically why it is disabled. `detail` is what the row stands for, as structure; see *Detail*, at the end of this chapter. |
| `empty` | Text shown in place of the rows when there are none. |

Answer value: one enabled row's `value`. Values are compared as exact
JSON values.

A surface that cannot fit every column SHOULD drop columns from the
right rather than truncate every cell, so the daemon's column order
is also its priority order.

## `progress` — output

How far along the work is.

| State | Meaning |
|---|---|
| `value` | Progress so far. |
| `max` | The whole; absent means indeterminate. |

## `log` — output

Accumulating lines of detail.

| State | Meaning |
|---|---|
| `lines` | The lines so far. |
| `append` | *In a sparse patch only:* lines to add. |

`append` exists so progress detail need not resend the backlog; the
daemon folds appended lines into `lines` (§3.10), so a late joiner
receives the accumulated state.

## `action` — pressed

| State | Meaning |
|---|---|
| `validate` | Default true; false skips §3.9 validation. |
| `primary` | The action an unattended answerer presses. At most one per turn. |
| `destructive` | Pressing this destroys something; a surface SHOULD make it hard to press by accident. |

Answer value: none, ever. The pressed action travels in
`Answer.action`.

## Detail

A `table` row and a `text` element may each carry a `detail`: an
object of further facts about what the words stand for, as structure
rather than as text. A row's cells say "931.5 GiB" and nothing of what
is on the disk; its `detail` gives the size in bytes and the
partitions. A sentence says the whole disk will be erased, Windows
included; its `detail` gives the disk it is about.

It exists for a surface that knows the page by its template id (§3.2)
and draws it as more than its elements. Its keys are the daemon's,
documented with the daemon's refs; this chapter defines none. A
surface that does not know them MUST ignore `detail`, and a daemon
MUST NOT put there anything a person needs in order to answer: a row's
cells and note, and a text's words, remain everything they say to a
surface that draws them as they are.
