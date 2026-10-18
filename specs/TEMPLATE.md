# SPEC-NNNN: <topic>

- Answers: R-NNNN
- Status: Draft | Approved (user, YYYY-MM-DD)
- Disc build: SCUS-97367 v1.00 (state any fingerprint the clean side should check)
- Confidence: High | Medium | Low, per section if mixed

## Summary

One paragraph: what this is and what the clean side can build with it.

## Where it lives on the disc

ISO file, archive resource, or ELF data address the runtime reads it from. Never paste the data itself.

## Layout

Field tables: offset, size, type, endianness, meaning, valid range. Describe variable-length parts and how to find their end.

## Behavior

Plain language and math. Describe *what* happens and in what order where order matters, not how the original code is structured.

## Edge cases and error handling

Malformed/out-of-range values, what the original does with them if known.

## Test vectors

Small hand-checkable input -> output examples, or synthetic byte sequences that are not copied from the disc.

## Open questions

What is unknown or guessed. The clean side must not fill these gaps by inspecting the original.

<!--
Forbidden in specs: decompiled/disassembled code or line-by-line pseudo-code, decompiler names (FUN_/DAT_/LAB_),
code addresses, copied comments, hex dumps, extracted assets, copied data tables, research-side source code.
Evidence for how this was learned stays in ../neopets-research/notes.
-->
