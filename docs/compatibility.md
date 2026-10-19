# MSIP compatibility

This migration retains crate version 0.1.1 and wire version 1. It is not a
release or an instruction to change downstream revision pins. Cargo remains
supported, and valid message headers and JSON field names are unchanged.

The quality pass makes these changes:

- `StateError::Malformed(String)` reports malformed sparse updates. Exhaustive
  Rust matches must handle this added variant.
- Rejected sparse batches no longer partially modify daemon or surface state.
  Malformed End bodies leave the surface page intact.
- Built-in inputs require the specified JSON encoding. Select and table answers
  must use offered values; table rows with `enabled: false` are rejected.
- Output elements reject answers and do not contribute required-field errors
  or defaults. Extension-specific validation remains with the embedding daemon.
- Required inputs need explicit answer values even when they have defaults.
  Optional defaults still apply; `validate: false` still skips required checks.
- Frame reads and writes reject non-object JSON with `FrameError::Json`.
  Invalid outgoing bodies fail before any header bytes are written.

Applications that depended on invalid messages must correct their encodings.
The library still leaves domain validation, peer identity, authorization,
capability negotiation, and connection-state enforcement to its caller.

The original history is retained in an external bundle and archive refs during
message normalization. Commit authorship, timestamps, trees, and substantive
historical facts are preserved. Rewritten history needs an explicitly approved
push; existing downstream pins are not silently replaced.
