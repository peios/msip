---
title: Scope and Roles
description: What PGSS MSIP specifies and who its parties are — a daemon driving generic UI surfaces through a conversation of pages, and the boundaries between them.
---

This chapter specifies **PGSS MSIP**, the Multi-Surface Interaction
Protocol: the protocol by which a daemon or long-running operation
drives one or more generic user-interface *surfaces* — showing
information, reporting progress, and requesting input — without
knowing or caring how any of them renders.

Two roles participate.

The **daemon** owns the work and the decisions. It offers one or more
*conversation kinds* on a socket, issues the conversation's pages,
decides what every answer means, and decides what page follows. Only
the daemon knows the shape of the conversation.

The **surface** renders. It draws the elements it is sent, collects
the person's input, and returns answers without interpreting them. A
TUI, a graphical installer, a web page and an automated agent are all
surfaces, and the daemon cannot tell them apart except by what they
declare they can render.

The person at the surface is not a party to the conversation, in the
same sense as the principal of a logon (§2.1).

Both roles are publicly implementable. A third party MAY ship a new
surface for an existing daemon, a new daemon driven by existing
surfaces, or both, and interoperate with the other half unchanged.
That substitution is the point of the protocol: a daemon that offers a
conversational interface to generic surfaces MUST offer it as
specified here, precisely so that surfaces need not be written per
daemon.

This chapter covers:

- the shape of a conversation: who opens it, who drives it, how many
  surfaces may attach, and how concurrent answers are arbitrated (§3.3)
- what each role must establish for itself rather than believe from a
  message, and the access control governing conversations (§3.4)
- message framing and the rules under which the format may be
  extended (§3.5)
- how a connection binds to a conversation (§3.6)
- turns, elements, answers, and mid-turn state (§3.7 to §3.10)
- how a conversation ends, and what a protocol error is (§3.11)
- the obligations binding on each role (§3.12)

This chapter does not cover:

- What any conversation is *about*. The pages of a particular daemon —
  an installer's disk selection, a first-boot flow's account page —
  are that daemon's specification, not this chapter's.
- How a surface renders any element. Rendering is deliberately
  unspecified; §3.8 states the little a surface owes the daemon.
- Unattended operation. A machine-driven answering agent is a surface
  like any other; the policy by which it answers is outside MSIP.
