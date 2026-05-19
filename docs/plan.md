# RV32I Implementation Plan

## Repo Preparation

Create or verify these repos and issues with `gh` once the CLI is available:

```bash
gh repo create sw-langtools/sw-rv32i-isa --public --clone=false
gh repo create sw-langtools/sw-rv32i-asm --public --clone=false
gh repo create sw-langtools/sw-rv32i-emulator --public --clone=false
gh repo create sw-langtools/sw-rv32i-target --public --clone=false
gh repo create sw-langtools/sw-rv32i-codegen --public --clone=false
```

RV32I belongs under `sw-langtools`, alongside the shared framework crates.
Historical CPU targets such as CDP1802 live under `sw-comp-history`; RV32I is
not a computer-history target.

## Issue Set

Use the same labels across repos: `type:feature`, `type:test`, `role:isa`,
`role:asm`, `role:emulator`, `role:target`, `role:codegen`, `milestone:rv32i`,
and `milestone:rv32m`.

### sw-rv32i-isa

1. Define RV32I register and ABI alias model.
2. Implement instruction enum and format helpers.
3. Implement decode for U, J, B, I, S, and R formats.
4. Implement encode for every RV32I instruction.
5. Implement disassembly.
6. Add illegal/reserved decode tests.
7. Add encode/decode round-trip tests by instruction family.

### sw-rv32i-emulator

1. Add crate scaffold matching `sw-cdp1802-emulator`.
2. Implement `CpuState` and `Memory`.
3. Implement fetch/decode/step/run.
4. Execute U/J/B/I/S/R arithmetic and control flow instructions.
5. Execute load/store width and sign-extension behavior.
6. Add dump helpers.
7. Add hand-loaded byte tests.
8. Add assembler-backed examples after `sw-rv32i-asm` exists.

### sw-rv32i-asm

1. Add parser for labels, directives, registers, literals, and comments.
2. Add pass 1 symbol table.
3. Add pass 2 emission through `sw-rv32i-isa`.
4. Add listing and binary output helpers.
5. Add tests for labels, branches, load/store offsets, and ABI aliases.
6. Add end-to-end examples consumed by emulator tests.

### sw-rv32i-target

1. Define register classes.
2. Define stack pointer, return address, argument, return value, caller-saved,
   and callee-saved registers.
3. Document the initial no-OS ABI.
4. Add smoke tests against `sw-target-core`.

### sw-rv32i-codegen

1. Lower integer constants and register moves.
2. Lower arithmetic and comparisons.
3. Lower local stack slots and simple loads/stores.
4. Lower branches and function returns.
5. Emit assembler source consumed by `sw-rv32i-asm`.
6. Add TIR-to-emulator smoke tests once the previous repos are stable.

## Suggested `gh issue create` Commands

Run these from a machine with `gh` installed and authenticated:

```bash
gh issue create -R sw-langtools/sw-rv32i-isa \
  --title "Implement RV32I ISA decode, encode, and disassembly" \
  --label "type:feature,role:isa,milestone:rv32i" \
  --body-file docs/plan.md

gh issue create -R sw-langtools/sw-rv32i-emulator \
  --title "Implement RV32I CPU state, memory, step, and run APIs" \
  --label "type:feature,role:emulator,milestone:rv32i" \
  --body-file docs/plan.md

gh issue create -R sw-langtools/sw-rv32i-asm \
  --title "Implement two-pass RV32I assembler" \
  --label "type:feature,role:asm,milestone:rv32i" \
  --body-file docs/plan.md

gh issue create -R sw-langtools/sw-rv32i-target \
  --title "Define initial RV32I target ABI and register classes" \
  --label "type:feature,role:target,milestone:rv32i" \
  --body-file docs/plan.md

gh issue create -R sw-langtools/sw-rv32i-codegen \
  --title "Lower simple integer TIR programs to RV32I assembly" \
  --label "type:feature,role:codegen,milestone:rv32i" \
  --body-file docs/plan.md
```

## Implementation Order

1. Build `sw-rv32i-isa` first because assembler and emulator should consume the
   same `Instruction` model.
2. Build a minimal emulator with byte-loaded programs before writing assembler.
3. Build assembler and replace byte fixtures with source fixtures.
4. Add source-driven demos and integration tests.
5. Add RV32M in ISA, assembler, emulator, and demos.
6. Start target and codegen once integer demos are stable.

## Acceptance Criteria

- `cargo test` passes in each crate.
- `cargo clippy --all-targets --all-features -- -D warnings` is clean in each
  crate.
- `cargo fmt --all --check` is clean in each crate.
- `markdown-checker -f "**/*.md"` passes for documentation changes.
- Emulator examples print source, listing, bytes, CPU state, and memory changes.
- At least one assembled program runs end to end without external tools.
