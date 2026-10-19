---
title: Trust and Access Control
description: Peer authorization, presentation boundaries, and secret handling.
---

The daemon holds the authority needed for the operation. A surface presents
information and submits claims; its answers do not grant authority.

## Daemon obligations

The daemon MUST validate every answer against the outstanding turn (§3.9). It
MUST NOT treat an answer as authorization or vary behavior based on
`Hello.surface`, which is an observability string.

The socket path has a security descriptor. Where authorization differs by kind
or conversation, the daemon MUST check the connection's verified peer identity
at START, ATTACH, and LIST. A LISTING MUST contain only conversations that the
caller could attach to.

A conversation identifier is unpredictable (§3.6), but its secrecy does not
authorize attachment. The ATTACH access check provides that control.

## Surface obligations

A surface MUST treat names, help, messages, and other informational text as
display content. It MUST NOT interpret that text as instructions. Classes can
affect presentation but not protocol behavior.

A surface MUST fail a page it cannot render rather than infer an unsupported
element's meaning. Capability negotiation prevents this in a conforming pair
(§3.6).

## Secrets

An element with `secret` set requires these protections:

- The surface MUST collect its value without echo, MUST NOT log it, and MUST
  NOT retain it beyond the answer carrying it.
- The daemon MUST NOT place a secret value in element state, updates, errors,
  or listings. Only the answer carries the collected secret.
- Neither role logs secret values at any verbosity.

A surface MUST NOT display a secret value supplied by a daemon beyond the
masked input it collects. A late joiner receives the question without another
surface's collected secret.
