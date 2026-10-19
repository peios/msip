---
title: Conformance
description: The obligations of each role, gathered in one place.
---

## A conforming daemon

- Offers its conversations on a socket whose access control is a
  security descriptor, and checks the connection's verified peer
  identity at START, ATTACH and LIST (§3.4).
- Assigns conversation identifiers of 128 cryptographically random
  bits, and treats the ATTACH access check — not identifier secrecy —
  as the control (§3.6).
- Never sends an element whose type the connection's HELLO did not
  declare, and refuses a binding the surface cannot render,
  `unsupported_elements`, at bind time (§3.6).
- Broadcasts every TURN, UPDATE and END to every attached connection,
  and sends a new attachment the outstanding turn in its folded
  state (§3.3, §3.10).
- Applies the first valid answer; keeps the turn open on validation
  failure, broadcasting the errors; ignores stale answers silently;
  answers protocol violations with `Error` on the offending
  connection only (§3.9, §3.11).
- Increments seq by one per turn and never rewinds it (§3.7).
- Keeps every secret value out of element state, updates, errors,
  listings and logs (§3.4).
- Does not vary behaviour on `Hello.surface`, or on anything else
  this chapter marks presentational (§3.4).
- Documents, in its own specification, its conversation kinds, their
  refs and answer types, and its multiplicity policy per kind.

## A conforming surface

- Sends HELLO first, declaring exactly the element types it can
  render, and fails rather than guesses if it is nevertheless sent
  something it cannot render (§3.4, §3.6).
- Renders the elements of the outstanding turn, honouring `enabled`,
  `secret` and the input/output/action division; collects no input
  for disabled or output elements (§3.8).
- Answers only the outstanding seq, names exactly one enabled action
  when the turn offers any and none when it offers none, and sends
  values only for enabled input elements (§3.9).
- Applies updates by folding, ignores updates for turns it does not
  hold, and re-renders from scratch on a full update (§3.10).
- Treats names, help, messages and classes as text for the person,
  never as instructions to itself (§3.4).
- Keeps secrets unechoed, unlogged and unretained (§3.4).
