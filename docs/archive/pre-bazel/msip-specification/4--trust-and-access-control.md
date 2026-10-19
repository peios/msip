---
title: Trust and Access Control
description: The daemon is privileged and the surface is not — what each role must not trust, who may attach to a conversation, and how secrets are kept off the wire they don't belong on.
---

## The asymmetry

MSIP exists to separate privilege from presentation. The daemon
typically holds the authority the work requires; the surface renders,
and holds none. The protocol is designed so that nothing a surface
sends can be more than a claim, and nothing a daemon sends obliges a
surface to do more than draw.

## What the daemon must not trust

Everything a surface sends is input from an untrusted peer.
Specifically, a daemon:

- MUST NOT treat any answer as authorisation. Whether the peer may do
  what the answer asks is established by the access decision of §3.4,
  made from the connection's verified peer identity — never from the
  content of a message.
- MUST NOT vary its behaviour on `Hello.surface` (§3.6). It is an
  observability string.
- MUST validate every answer against the outstanding turn (§3.9)
  rather than assume a well-behaved surface.

## What the surface must not trust

A surface renders what it is sent, but:

- MUST NOT render a secret element's value back to the person beyond
  the masked input it collects (§3.8), whatever the daemon sends.
- MUST NOT interpret informational text, and MUST NOT vary its
  behaviour on the content of names, help text, messages, or classes.
  Text is for the person; behaviour follows structure only.
- MUST fail a page it cannot render rather than guess. The
  negotiation of §3.6 makes this unreachable in a conforming pair;
  the rule is what a surface does when it is reached anyway.

## Who may connect, and who may attach

Access to the socket is governed by a security descriptor on the
socket path, like any object. Offering separate conversation kinds to
separately-trusted callers on one socket is the daemon's problem;
where the difference matters, the daemon MUST check the connection's
verified peer identity per kind at START, and per conversation at
ATTACH and LIST.

A conversation's identifier is unguessable (§3.6), but secrecy of the
identifier is not the control: the access check at ATTACH is. A
LISTING MUST contain only conversations the caller could attach to.

## Secrets

An element with `secret` set collects material that must not escape:

- A surface MUST collect it without echo, MUST NOT log it, and MUST
  NOT retain it beyond the answer that carries it.
- A daemon MUST NOT place a secret value in any element state, in any
  UPDATE, or in any error text — the answer is the only message that
  ever carries it. A late joiner therefore never receives another
  surface's secret, because the outstanding turn it is sent contains
  only the ask.
- Neither role logs a secret value under any verbosity.
