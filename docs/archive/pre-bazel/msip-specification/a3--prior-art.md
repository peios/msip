---
title: Prior Art
description: Where MSIP sits against PAM and PGSS Logon, Debconf, Subiquity and the desktop agent protocols — and which resemblances are deliberate.
---

## PAM, via PGSS Logon

The conversation shape — a privileged party that asks for what it
needs, an untrusted renderer that collects without understanding, and
capability negotiation so nothing unrenderable is ever sent — is
PAM's, adopted into this document once already by Logon (§2). Logon's
credential exchange is in this sense a specialisation of MSIP: a
single-kind, single-surface conversation whose elements are prompts.
The two chapters remain separate protocols on separate sockets — a
logon conversation is one connection with no attachment, no
persistence and no broadcast, and gains from that austerity (§2.3) —
but a reader will recognise the shape, and divergence between the two
should be deliberate.

## Debconf

Debconf solved this chapter's exact problem — one engine, many
frontends — and proved the central discipline: a tiny, semantic
vocabulary of question types, with rendering wholly owned by the
frontend. It also demonstrates the failure mode this chapter's §3.7
legislates against: its templates grew presentation-shaped extensions
the moment a frontend could exploit them. What MSIP does not adopt is
the shared question database and its state machine; a conversation is
owned by its daemon, not by a global store.

## Subiquity

Ubuntu's installer is the closest modern relative: a privileged
server driving replaceable clients (TUI, web) over a wire protocol,
with attach-and-resume. It validates the process model — and warns
about the API: its per-screen endpoints make each page a bespoke
contract, so every new page changes both sides. MSIP's turn/element
model is the generalisation that keeps the wire contract fixed while
pages change.

## Desktop agent protocols

polkit's authentication agents and systemd's ask-password watchers
are the daemon-has-a-need pattern: a session registers a renderer,
and privileged software asks through it. §3.3's agent-style
conversation is that pattern expressed without a second protocol —
the surface still opens; the daemon still drives.

## Design influences

**Broadcast with first-valid-answer arbitration** is what a
conversation that outlives connections forces: once two surfaces can
watch, choosing a "driving" surface means takeover semantics and
detach protocols; arbitration by seq needs neither, and turn sequence
numbers were already required for staleness.

**Refs versus seqs** — semantic identity apart from sequence — is
the split that keeps MSIP a conversation rather than a form language.
A stable question list would be a schema; stable refs on an
unpredictable path is a dialogue.
