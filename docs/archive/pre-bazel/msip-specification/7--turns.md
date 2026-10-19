---
title: Turns
description: The page the daemon issues — seq, template id, name and class — and the rules for turns that expect no answer.
---

## Issuing

The daemon advances the conversation by broadcasting `Turn`:

| Field | Meaning |
|---|---|
| `seq` | This turn's sequence number. |
| `id` | Optional template identity (§3.2). |
| `name` | Optional display title. |
| `class` | Optional presentation hints. |
| `elements` | The page, in order. |

`seq` starts at 1 and increments by exactly 1 per turn of a
conversation. Issuing a turn implicitly closes the previous one:
answers to it become stale (§3.3). A daemon MUST NOT reuse or rewind
a seq within a conversation.

The elements are an ordered list, and the order is the daemon's one
presentational statement: it MUST order elements helpfully, and a
surface SHOULD present them in that order — but MAY depart from it,
and a daemon MUST NOT depend on the order being visible. Everything
else about layout belongs to the surface. This protocol deliberately
has no vocabulary for columns, grouping, emphasis or geometry, and
anyone extending it SHOULD refuse the temptation: an element says
what is being asked, never how it looks.

## Turns that expect no answer

A turn with no enabled input elements and no enabled actions expects
no answer: the daemon advances past it on its own — typically a
progress page, updated in place (§3.10) until the work completes and
the next turn or the END follows. A surface MUST NOT be required to
acknowledge such a turn, and the daemon owes it nothing before
moving on.

A turn that does expect an answer stays outstanding until a valid
answer is applied (§3.9) or the daemon supersedes it — the daemon
remains free to issue a new turn at any time, because the flow is
its own.
