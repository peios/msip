---
title: Turns
description: Sequence numbers, ordered elements, and progress-only turns.
---

The daemon advances a conversation by broadcasting Turn. The message describes
the complete page and closes the preceding turn.

## Fields and sequence

Turn has these fields:

| Field | Meaning |
| --- | --- |
| `seq` | Sequence number |
| `id` | Optional template identity (§3.2) |
| `name` | Optional display title |
| `class` | Optional presentation hints |
| `elements` | Ordered element list |

Sequence numbers start at 1 and increase by exactly 1 for each turn. The
daemon MUST NOT reuse or rewind a sequence number within a conversation.
Answers to a superseded turn are stale (§3.3).

The daemon MUST order elements helpfully, and the surface SHOULD retain that
order. A surface MAY rearrange them; the daemon MUST NOT depend on their
visible order. Layout, grouping, and geometry belong to the surface. Protocol
extensions SHOULD retain this separation.

## Turns without answers

A turn with no enabled inputs or actions expects no answer. The daemon
advances it independently, often updating a progress page until the work
completes (§3.10). A surface MUST NOT be required to acknowledge such a turn.

A turn expecting an answer remains outstanding until a valid answer is applied
or the daemon supersedes it. The daemon can issue a replacement turn at any
time.
