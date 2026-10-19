---
title: Design Background
description: The design relationships behind independent surfaces,
  persistent conversations, and semantic element references.
---

The multi-surface interaction protocol (MSIP) separates operation authority
from presentation. The daemon describes questions and outputs through a
shared element vocabulary, so a different renderer can implement the same
contract.

## Related protocols

Logon (§2) also separates a privileged requester from an untrusted renderer.
It uses a single connection without attachment, persistence, or broadcast.
MSIP retains a separate protocol because its conversations can outlive
connections and support several attached surfaces.

The original design review also considered PAM, Debconf, Subiquity, polkit,
and systemd password agents. The repository archive retains that review.
Those comparisons are design background, not conformance requirements or
claims that the protocols are interchangeable.

## Arbitration and identity

Broadcast with first-valid-answer arbitration avoids assigning a controlling
surface. Sequence numbers identify the outstanding turn and make competing
late answers stale. The protocol needs no takeover or detach negotiation.

Stable element refs identify questions independently of the path through a
conversation. Template ids identify compatible page shapes. Neither promises
a fixed sequence of pages or determines what an action means.
