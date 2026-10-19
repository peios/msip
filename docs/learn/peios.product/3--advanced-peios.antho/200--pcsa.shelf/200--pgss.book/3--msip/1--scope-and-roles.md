---
title: Scope and Roles
description: The protocol roles, interoperability requirements, and scope of
  MSIP.
---

The multi-surface interaction protocol (MSIP) lets a daemon drive one or more
user interfaces without depending on their rendering technology. It carries
information, progress, and requests for input as a conversation of pages.

The **daemon** owns the operation and its decisions. It offers conversation
kinds, issues turns, interprets answers, and chooses the next turn. The
**surface** renders elements, collects input, and returns answers. A terminal,
graphical application, web client, or automated client can act as a surface.

The person using a surface is not a protocol party, as with the principal of a
logon (§2.1). Both protocol roles are publicly implementable. A third party
MAY implement either role independently. A daemon offering a conversational
interface to generic surfaces MUST follow this chapter.

This chapter specifies the following contracts:

- Conversation ownership, persistence, and answer arbitration (§3.3).
- Trust, access control, and secret handling (§3.4).
- Framing and extensibility (§3.5).
- Greeting and binding (§3.6).
- Turns, elements, answers, and updates (§3.7 to §3.10).
- Completion, errors, and conformance (§3.11 and §3.12).

The daemon's specification defines its conversation kinds, refs, answer
meanings, and multiplicity policy. This chapter does not prescribe element
layout or an automated surface's answering policy.
