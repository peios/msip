---
title: Conformance
description: The responsibilities required of daemons and surfaces.
---

Conformance requires the complete protocol, including authorization and
transport responsibilities outside a state-helper library.

## Daemons

A conforming daemon satisfies these obligations:

- Protect its socket with a security descriptor and authorize START, ATTACH,
  and LIST using verified peer identity (§3.4).
- Assign identifiers from 128 cryptographically random bits and authorize
  attachment independently of identifier secrecy (§3.6).
- Send only declared element types and refuse unsupported bindings (§3.6).
- Broadcast turns, updates, and completion to all attached connections;
  provide folded state to late joiners (§3.3 and §3.10).
- Apply the first valid answer, retain turns after validation failure, ignore
  stale answers, and limit protocol errors to the offending connection (§3.9).
- Increase sequence numbers without reuse and keep secrets out of state,
  updates, errors, listings, and logs (§3.4 and §3.7).
- Document conversation kinds, refs, answer encodings, and multiplicity; keep
  presentation strings separate from behavior.

## Surfaces

A conforming surface satisfies these obligations:

- Send HELLO first with its supported element types and reject pages it cannot
  render (§3.6).
- Respect input, output, action, availability, and secret-handling rules
  (§3.8).
- Answer only the outstanding sequence with permitted actions and input values
  (§3.9).
- Fold sparse updates, replace full updates, and ignore updates for other
  turns (§3.10).
- Treat informational text as display content and keep secrets masked,
  unlogged, and unretained (§3.4).
