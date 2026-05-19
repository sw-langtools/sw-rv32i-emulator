# RV32I Emulator PRD

## Problem

The Software Wrighter language tools need a small, teachable RISC-V target that
can run compiler and assembler output in Rust and later in WASM demos. The
research notes point to RISC-V as the fastest path for a modern embedded target
because the base ISA is fixed-width, modular, and can start without floating
point, vector, compressed, or OS compatibility complexity.

## Users

- Toolchain developer validating Tiny C, Pascal, or Lisp compiler output.
- Demo author embedding an emulator in browser-based live coding examples.
- Learner inspecting how source becomes assembly, bytes, and CPU state.

## Goals

- Implement a deterministic RV32I emulator core in Rust.
- Reuse the split already used by `sw-cdp1802-*`.
- Support assembler-to-emulator demos without external RISC-V tools.
- Keep the first milestone small enough to finish before target and codegen
  work begins.
- Leave a clear path to RV32M once RV32I is stable.

## Non-Goals

- Running Linux binaries.
- Full RISC-V privileged architecture.
- Floating point ABI or FP instruction execution.
- Vector or compressed instruction support.
- Cycle-accurate SoC or board emulation.
- GCC/LLVM compatibility as a first milestone.

## Functional Requirements

- Fetch and execute aligned 32-bit RV32I instructions from little-endian memory.
- Implement all RV32I integer register-register, immediate, branch, jump, load,
  store, `LUI`, `AUIPC`, `FENCE`, `ECALL`, and `EBREAK` decode cases.
- Execute practical user-mode integer subsets, including arithmetic, compare,
  branch, jump, load, and store instructions.
- Treat `x0` as immutable zero.
- Expose `step` and bounded `run` APIs.
- Provide human-readable CPU state and memory dump helpers.
- Provide examples that assemble source from `examples/asm/*.s`, run it, and
  print source, listing, bytes, final CPU state, and memory changes.

## Quality Requirements

- All behavior must be covered by Rust tests.
- ISA decode should include format-level coverage and reserved/illegal cases.
- Emulator tests should cover edge cases such as sign extension, branch target
  calculation, `x0` writes, `JALR` low-bit clearing, and load sign extension.
- Documentation must stay ASCII-only for existing markdown tooling.
- Public APIs should stay small and stable before WASM integration.

## Milestones

### M1: ISA Contract

`sw-rv32i-isa` defines registers, instruction enum, decode, encode, and
disassembly for RV32I.

### M2: Emulator Core

`sw-rv32i-emulator` executes enough RV32I to run hand-authored examples.

### M3: Assembler

`sw-rv32i-asm` assembles labels, register aliases, literals, directives, and the
RV32I mnemonic set.

### M4: End-to-End Demos

Examples demonstrate assemble, load, run, inspect, and listing output.

### M5: RV32M Extension

Add optional multiply/divide support after RV32I has reliable tests and demos.

### M6: Target and Codegen

Define calling convention and lower simple TIR integer programs to assembly.
