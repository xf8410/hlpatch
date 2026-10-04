# Portable collector publication boundary

The host creates one `Publisher` for a collector process. `Default` uses PID, Unix nanoseconds and an in-process counter; a host may instead call `new(id)` with its own unique process epoch. IDs are ASCII `[A-Za-z0-9._-]`, 1–128 bytes, excluding `.`, `..` and reserved `legacy-v1`.

The sole sampler calls `begin_capture` before reading current game state, then `finish_summary(ticket, raw, collector_version, captured_at_ms)` for legacy partial observations or `finish_capture(ticket, observation, captured_at_ms)` for an independently verified full state. HTTP and push only clone/read `current()`; they must never start another publication from a cached summary. A late ticket, foreign ticket, older timestamp or already numbered envelope is rejected without changing the current publication. Unchanged material retains its sequence and original timestamp.

`capabilities(version)` returns capability schema 1 and snapshot versions `[2]`, with the process `collector_instance_id`. A new process may reset its V2 sequence; this is never advertised as a decision-ready V1 stream. Older V1 consumers retain their existing source protocol independently.

Legacy summary conversion remains intentionally incomplete. It does not invent run identity, stage, internal turn, partner mapping, ingredient order, continuation or coherent capture evidence. `ready=false` and `missing_fields` prevent computation. Full capture remains blocked until these fields are verified from the game.

Validation: `cargo test --release --offline --locked --manifest-path ramen_observation/Cargo.toml`. Host tests verify immutable publication, duplicates, stale tickets, process identity, and explicit missing fields; they do not execute hooks or prove device memory semantics.
