# Protocol layered decode contract

## Core rule

Capture stage is provenance metadata, not an authorization or parsing gate.

- `request_plain` and `response_plain` are captured at verified game plaintext boundaries.
- Unity upload/download captures may be labelled `request_wire` or `response_wire`.
- A `wire` label **must not** disable probing, nested decoding, route extraction, class-name extraction, string scanning, comparison, or raw download.
- A heuristic result from wire bytes must not be represented as verified plaintext.

## Confidence labels

- `verified_plaintext_boundary`: bytes came from `HttpHelper.CompressRequest` input or `HttpHelper.DecompressResponse` output.
- `heuristic_candidate`: structure was inferred by non-destructive probing, including an offset or nested payload scan.

## Presentation rules

Compact endpoints:

- prefer structured MessagePack/UTF-8 output;
- never present an unknown raw payload as decrypted plaintext;
- cap inline strings, containers, recursion and hex preview;
- retain route/class/string findings when full decode fails;
- provide a raw capture reference for complete bytes.

Raw and archive endpoints remain available. Avoiding a large inline hex dump is not deletion and is not a prohibition on wire analysis.

## Anti-4444 behavior

A repeated-byte or opaque payload is returned with:

- `decoded: false`;
- its actual `source_stage`;
- `confidence: heuristic_candidate`;
- a bounded hex preview;
- repeated-byte diagnostics;
- extracted route, class-name and printable-string candidates;
- a hint/reference to the complete raw capture.

It must not be returned as a successful decrypted body.
