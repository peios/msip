---
title: Updates
description: Element state pushed within a turn — sparse and full, what each may change, and the folding rule that makes late joiners cheap.
---

The daemon changes the outstanding page without a new turn by
broadcasting `Update`:

| Field | Meaning |
|---|---|
| `seq` | The outstanding turn. |
| `full` | Replacement or patch. |
| `elements` | See below. |

A **sparse** update (`full` absent or false) carries patches: objects
naming an existing element by `ref` plus the fields that change — a
progress value, appended log lines, an `error`, an `enabled` flip, a
refreshed `choices`. A patch key set to `null` removes that field. A
patch MUST NOT name a ref absent from the turn, and MUST NOT carry a
secret element's collected value (§3.4).

A **full** update (`full: true`) carries complete elements and
replaces the turn's element list. It is the only way the *set* of
refs may change mid-turn. Changing the set is legal but SHOULD be
rare — a page that reshapes itself is usually two turns — and a
surface's obligation is only to re-render from scratch, which is why
a full update must be complete.

Updates are how progress happens: a working page is one turn with no
inputs (§3.7), updated as the work advances, so that every attached
surface — and any that attach midway — shows the same state.

## Folding

The daemon MUST fold every update into its stored copy of the
outstanding turn, and the turn it sends a late joiner (§3.6) is that
folded state. An update is therefore transient on the wire but never
lost: attaching and having-been-attached see the same page.

A surface receiving an update for a seq it does not hold MUST ignore
it — the TURN that supersedes it is already on the wire. A sparse
update naming a ref the surface does not know is a daemon defect; the
surface SHOULD fail the conversation rather than guess.
