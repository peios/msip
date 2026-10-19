---
title: Terminology
description: The nouns MSIP is specified in — conversation, turn, element, ref, action — and the identities each of them carries.
---

**Conversation.** One instance of a daemon driving one flow: a single
installation, one first-boot setup, one authorisation request. A
conversation belongs to the daemon, outlives any particular
connection, and is identified by an unguessable identifier the daemon
assigns.

**Kind.** The name of a flow a daemon offers — `install`, `oobe`. A
daemon lists its kinds at greeting time; a surface opens a
conversation by kind.

**Turn.** One page of the conversation. A turn is an ordered list of
elements issued as a unit, identified by its **seq** — a number the
daemon increments for every turn of a conversation. The seq is the
only identity a surface cites. A turn MAY also carry an **id**, a
stable template identity for surfaces that want to special-case a
known page: two turns with the same id MUST present the same set of
element refs and types, though their state may differ. An id is not
unique within a conversation and is never cited in a message.

**Element.** One item on a page: a piece of text, an input, a
progress report, an action. An element has a **type** from the
negotiated set (§3.6) and a **ref**.

**Ref.** The semantic identity of an element: what is being asked,
not where it appears. A ref is unique within its turn, stable across
turns and across runs of the same daemon, and documented by the
daemon's own specification. A ref recurs freely — a question re-asked
after a validation failure, or revisited because the person went
back, keeps its ref. Nothing about the *sequence* of turns is stable;
only refs and their answer types are.

**Action.** An element a surface presents as something to press
rather than something to fill in. Actions carry no value; an answer
names at most one, and the daemon alone knows what it means (§3.9).

**Attach.** Joining a connection to an existing conversation. Any
number of connections may be attached to one conversation at once;
every turn and update is broadcast to all of them.

**Class.** A presentation hint, on turns and on elements: a list of
tokens a surface MAY use to style what it renders. Classes promise
nothing and are never cited in a message.
