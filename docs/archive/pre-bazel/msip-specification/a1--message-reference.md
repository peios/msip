---
title: Message Reference
description: Every MSIP message — direction, type value, body fields — in one table per message.
---

Message type values. Surface-sent types have the high bit clear;
daemon-sent types have it set (`0x8000`).

| Message | Direction | `msg_type` |
|---|---|---|
| Hello | surface → daemon | `0x0001` |
| List | surface → daemon | `0x0002` |
| Start | surface → daemon | `0x0003` |
| Attach | surface → daemon | `0x0004` |
| Answer | surface → daemon | `0x0005` |
| Welcome | daemon → surface | `0x8001` |
| Listing | daemon → surface | `0x8002` |
| Bound | daemon → surface | `0x8003` |
| Refused | daemon → surface | `0x8004` |
| Turn | daemon → surface | `0x8005` |
| Update | daemon → surface | `0x8006` |
| Error | daemon → surface | `0x8007` |
| End | daemon → surface | `0x8008` |

Field tables. A field marked *opt* may be absent; its default is
given in the section that specifies the message.

**Hello** (§3.6): `element_types` (array of string), `surface` (opt
string).

**Welcome** (§3.6): `kinds` (array of string), `daemon` (opt string).

**List** (§3.6): no fields.

**Listing** (§3.6): `conversations` — array of `{conversation, kind,
seq}`.

**Start** (§3.6): `kind` (string).

**Attach** (§3.6): `conversation` (string, 32 lowercase hex).

**Bound** (§3.6): `conversation` (string), `seq` (number), `attached`
(opt bool, default false).

**Refused** (§3.6): `reason` (`access_denied` | `unknown_kind` |
`unsupported_elements` | `unavailable`), `message` (opt string).

**Turn** (§3.7): `seq` (number), `id` (opt string), `name` (opt
string), `class` (opt array of string), `elements` (array of element
objects, §3.8).

**Update** (§3.10): `seq` (number), `full` (opt bool, default false),
`elements` (array of objects: patches when sparse, complete elements
when full).

**Answer** (§3.9): `seq` (number), `action` (opt string), `values`
(opt object, ref → value).

**Error** (§3.11): `code` (`malformed` | `unbound` | `unknown_ref` |
`bad_value` | `unexpected_type`), `message` (opt string).

**End** (§3.11): `seq` (number), `outcome` (`complete` | `cancelled`
| `failed`), `message` (opt string).
