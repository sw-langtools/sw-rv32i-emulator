# RV32I Status

## Current State

Date: 2026-05-18

This repo currently contains planning and process documentation only. It does
not yet contain a Rust crate scaffold, `Cargo.toml`, source files, tests, or
examples.

The GitHub repo set has been created as public repos under `sw-langtools`:

- <https://github.com/sw-langtools/sw-rv32i-isa>
- <https://github.com/sw-langtools/sw-rv32i-asm>
- <https://github.com/sw-langtools/sw-rv32i-emulator>
- <https://github.com/sw-langtools/sw-rv32i-target>
- <https://github.com/sw-langtools/sw-rv32i-codegen>

Each repo has the shared labels from `docs/plan.md` and one top-level planning
issue:

- <https://github.com/sw-langtools/sw-rv32i-isa/issues/1>
- <https://github.com/sw-langtools/sw-rv32i-asm/issues/1>
- <https://github.com/sw-langtools/sw-rv32i-emulator/issues/1>
- <https://github.com/sw-langtools/sw-rv32i-target/issues/1>
- <https://github.com/sw-langtools/sw-rv32i-codegen/issues/1>

The local sibling repo review found:

- `../../sw-comp-history/sw-cdp1802-isa`: completed shared ISA crate with decode,
  encode, disassembly, and exhaustive tests.
- `../../sw-comp-history/sw-cdp1802-asm`: completed two-pass assembler that emits
  the shared ISA instruction model.
- `../../sw-comp-history/sw-cdp1802-emulator`: completed emulator with CPU state,
  memory, execution dispatch, demos, board models, and integration tests.
- `../../sw-comp-history/sw-ibm1130-*`: generated skeletons only.
- `../../sw-langtools/sw-isa-core`, `sw-target-core`, and `sw-codegen-core`:
  framework crates for shared traits and future target/codegen integration.

## Decisions Made

- Start with RV32I, not full RV64GCV.
- Keep RV32M as the first extension after RV32I is stable.
- Defer compressed, float, vector, privilege, and Linux compatibility work.
- Follow the CDP1802 repo split and crate dependency pattern.
- Make assembler and emulator share the ISA crate's `Instruction` model.

## Blockers

- The `.git` directory in this checked-out repo appears incomplete or mounted
  without normal git metadata, so `git status` does not work from this
  workspace.

## Next Step

Scaffold `sw-rv32i-isa` and implement the ISA contract before adding Rust source
to this emulator repo.

## Open Questions

- Should the emulator crate name remain `sw-rv32i-emulator` if RV32M is added,
  or should the docs describe the crate as RV32I base plus optional M extension?
- Should `ECALL` and `EBREAK` halt the CPU in demos or return typed trap errors
  immediately?
