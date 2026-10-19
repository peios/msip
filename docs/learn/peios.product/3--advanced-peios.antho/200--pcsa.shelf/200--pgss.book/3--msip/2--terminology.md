---
title: Terminology
description: Conversation identities, element references, and presentation
  hints.
---

The multi-surface interaction protocol (MSIP) distinguishes operation
identity, turn sequence, and element meaning.

**Conversation.** One instance of a flow, such as installation, first-boot
setup, or authorization. It belongs to the daemon and outlives individual
connections. The daemon assigns its identifier.

**Kind.** A flow name offered by a daemon, such as `install` or `oobe`. The
greeting lists available kinds.

**Turn.** One page, represented by an ordered element list. Its `seq`
increases with every turn and is the only turn identity cited in answers. A
turn MAY carry a stable template `id`. Turns with the same `id` MUST have the
same element refs and types, although their state can differ. A template id
need not be unique within a conversation.

**Element.** One item on a page, identified by a `ref` and an element `type`
from the negotiated set (§3.6).

**Ref.** An element's semantic identity. It is unique within a turn and stable
across turns and runs of the daemon. Repeating a question retains its ref,
including after validation failure or navigation. The daemon documents refs
and answer encodings; it does not promise a fixed turn sequence.

**Action.** A selectable element with no answer value. An answer names at most
one action. The daemon defines its meaning (§3.9).

**Attach.** Bind a connection to an existing conversation. Multiple
connections can attach simultaneously and receive the same broadcasts.

**Class.** A presentation token on a turn or element. A surface MAY use
classes for styling. Classes make no behavioral promise and are not cited in
answers.
