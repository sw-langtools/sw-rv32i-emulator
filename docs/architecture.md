# RV32I Toolchain Architecture

## Purpose

This project is the emulator role in a small RISC-V toolchain family. It
should follow the split used by the recent CDP1802 work:

- `sw-rv32i-isa`: instruction model, encode, decode, disassembly
- `sw-rv32i-asm`: two-pass assembler that emits ISA instructions and bytes
- `sw-rv32i-emulator`: CPU state, memory, execution, demos
- `sw-rv32i-target`: ABI, calling convention, register classes
- `sw-rv32i-codegen`: TIR lowering to RV32I or RV32IM assembly
- `demos` or live-demo repos: WASM/browser integration after the Rust core is
  useful from examples and tests

The emulator depends on `sw-rv32i-isa` and `sw-isa-core`. Examples and
integration tests may depend on `sw-rv32i-asm`, matching
`sw-cdp1802-emulator`.

## Scope

The first executable target is RV32I in machine-like flat memory. RV32M should
be treated as a follow-up milestone because multiply and divide are valuable
for Tiny C, but not required for the first decode/execute loop.

Excluded from the first milestone:

- privilege modes, CSRs, traps, and interrupts beyond explicit illegal
  instruction reporting
- Linux ABI compatibility
- floating point extensions
- compressed 16-bit instructions
- atomics and vectors
- MMU, devices, timers, and platform firmware

## Crate Boundaries

`sw-rv32i-isa` owns the architectural instruction vocabulary:

- register identifiers `x0..x31` plus ABI aliases
- immediate formats and sign extension rules
- 32-bit instruction word decode and encode
- canonical disassembly text
- exhaustive tests for opcode and funct coverage

`sw-rv32i-asm` owns source syntax:

- labels, `ORG`, comments, decimal and hexadecimal literals
- register aliases
- pseudo-instructions only when they expand deterministically
- output helpers for raw binary, Intel HEX or flat hex, and listings

`sw-rv32i-emulator` owns runtime state and semantics:

- `CpuState` with 32 integer registers, program counter, halt flag, and
  instruction count
- `Memory` as little-endian byte-addressable RAM
- `step` and `run` APIs
- execution dispatch over `sw_rv32i_isa::Instruction`
- focused demos assembled from sibling assembler sources

`sw-rv32i-target` and `sw-rv32i-codegen` should remain skeletons until the ISA,
assembler, and emulator have stable enough contracts for end-to-end integer
program demos.

## Execution Model

The emulator fetches one aligned 32-bit little-endian instruction from `pc`,
decodes it with `sw-rv32i-isa`, advances `pc` by 4, and executes the
instruction. Branches and jumps overwrite `pc` after the default increment.

Register `x0` is hardwired to zero. All writes to `x0` are ignored. Arithmetic
uses wrapping 32-bit behavior. Loads and stores operate on byte-addressed
memory with RV32I little-endian layout. Misaligned accesses should initially
return an execution error rather than silently emulating hardware-specific
behavior.

## Public API Shape

The emulator should resemble:

```rust
pub mod dump;
pub mod exec;
pub mod memory;
pub mod state;

pub use dump::{format_cpu_state, format_hex_dump};
pub use exec::{ExecError, run, step};
pub use memory::Memory;
pub use state::CpuState;
```

`ExecError` should distinguish at least:

- halted CPU
- decode error
- illegal instruction
- memory access out of bounds
- misaligned instruction fetch or data access

## Testing Strategy

Build from small vertical slices:

1. ISA decode/encode round trips for a representative instruction from each
   RV32I format.
2. Emulator unit tests for register zero, arithmetic, branches, jumps, loads,
   and stores.
3. Assembler tests that prove labels and immediates produce the same bytes as
   the ISA encoder.
4. End-to-end examples that assemble a program, load it into memory, run it,
   and inspect registers and memory.

The CDP1802 repos show the desired pattern: exhaustive ISA tests in the ISA
crate, parser and listing tests in the assembler, and demo-driven integration
tests in the emulator.
