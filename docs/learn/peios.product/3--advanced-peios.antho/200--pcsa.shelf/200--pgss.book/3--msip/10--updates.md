---
title: Updates
description: Sparse and full element updates and consistent folded state.
---

The daemon broadcasts Update to change the outstanding page without issuing
another turn. This supports progress, log output, and validation errors.

Update has these fields:

| Field | Meaning |
| --- | --- |
| `seq` | Outstanding turn sequence |
| `full` | Whether the update replaces the element list |
| `elements` | Complete elements or sparse patches |

## Sparse updates

With `full` absent or false, each patch names an existing element by `ref` and
supplies changed fields. A key with value `null` removes that field. A patch
MUST NOT name an absent ref or contain a secret element's collected value
(§3.4).

Typical changes include progress values, appended log lines, errors,
availability, and refreshed choices. Log `append` entries fold into `lines`
(§3.B).

## Full updates

With `full: true`, complete elements replace the element list. This is the
only way to change the set of refs within a turn. Such changes SHOULD be rare;
a surface is only required to render the replacement from scratch.

## Folding

The daemon MUST fold each update into its stored turn. An attaching surface
receives this state rather than an update history (§3.6).

A surface MUST ignore updates for sequence numbers it does not hold. A sparse
update with an unknown ref indicates a daemon defect; the surface SHOULD fail
the conversation rather than infer a target.
