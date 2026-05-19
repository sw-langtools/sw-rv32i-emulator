# RV32I Emulator Design

## Reference Pattern

The CDP1802 repos provide the strongest local precedent:

- `sw-cdp1802-isa` owns `Instruction`, register parsing, decode, encode, and
  exhaustive opcode tests.
- `sw-cdp1802-asm` is a two-pass assembler that emits
  `sw_cdp1802_isa::Instruction` rather than a parallel opcode model.
- `sw-cdp1802-emulator` keeps state, memory, board hooks, step/run dispatch,
  examples, and integration tests in the emulator crate.

The RV32I implementation should preserve that ownership model.

## State

`CpuState`:

```rust
pub struct CpuState {
    regs: [u32; 32],
    pc: u32,
    pub halted: bool,
    pub instr_count: u64,
}
```

Recommended methods:

- `new()`
- `pc()`
- `set_pc(value)`
- `advance_pc(bytes)`
- `read_reg(reg)`
- `write_reg(reg, value)` with ignored writes to `x0`
- `regs()` for dump and tests

Avoid exposing mutable register arrays directly. Keeping `x0` enforcement in
one write path prevents repeated bugs.

## Memory

Use byte-addressable `Vec<u8>` memory for the first milestone.

Recommended methods:

- `new(size)`
- `load(addr, bytes)`
- `read_u8`, `read_u16`, `read_u32`
- `write_u8`, `write_u16`, `write_u32`
- signed load helpers may live in execution code if that keeps memory simple

All multi-byte accesses are little-endian. Bounds and alignment errors should
return typed errors.

## Decode and Instruction Model

`sw-rv32i-isa` should model decoded instructions by semantic operands, not raw
bit fields:

```rust
pub enum Instruction {
    Lui { rd: Reg, imm: i32 },
    Auipc { rd: Reg, imm: i32 },
    Jal { rd: Reg, offset: i32 },
    Jalr { rd: Reg, rs1: Reg, offset: i32 },
    Branch { cond: BranchCond, rs1: Reg, rs2: Reg, offset: i32 },
    Load { width: LoadWidth, rd: Reg, rs1: Reg, offset: i32 },
    Store { width: StoreWidth, rs1: Reg, rs2: Reg, offset: i32 },
    OpImm { op: ImmOp, rd: Reg, rs1: Reg, imm: i32 },
    Op { op: RegOp, rd: Reg, rs1: Reg, rs2: Reg },
    Fence,
    Ecall,
    Ebreak,
}
```

This keeps the emulator dispatch readable and keeps assembler/codegen from
duplicating RISC-V bit layout rules.

## Execute Semantics

The step algorithm:

1. Return `ExecError::Halted` if the CPU is halted.
2. Fetch a 32-bit word at `pc`.
3. Decode through `sw-rv32i-isa`.
4. Set `next_pc = pc + 4`.
5. Execute instruction, mutating registers and memory.
6. Write `next_pc` unless the instruction changed control flow.
7. Increment `instr_count`.

`ECALL` and `EBREAK` should halt or return explicit trap errors in M1. Choose
one behavior and test it. A simple `halted = true` behavior is enough for demos,
but typed trap errors will be easier to extend.

## Assembler Syntax

Start with GNU-like RISC-V source where practical:

```asm
ORG 0x0000
start:
  addi x1, x0, 5
  addi x2, x0, 7
  add  x3, x1, x2
  sw   x3, 0x100(x0)
  ebreak
```

Required parser features:

- semicolon and hash comments if both are easy; otherwise document one comment
  form first
- labels and forward references
- `ORG`
- decimal and `0x` literals
- `x0..x31`
- ABI aliases such as `zero`, `ra`, `sp`, `a0`, and `t0`
- offset addressing for loads and stores

Pseudo-instructions should wait until base mnemonics are stable.

## Demo Contract

Examples should be source-driven, not byte-array-driven:

- source lives in `examples/asm/*.s`
- Rust examples use `include_str!`
- assembler output is printed or available behind flags
- memory load uses assembled bytes
- final output includes CPU state and changed memory

This mirrors the CDP1802 demos and makes regressions easy to inspect.

## WASM Readiness

Do not add WASM bindings during the first emulator core milestone. Keep the core
state serializable by design:

- plain integer registers
- explicit memory byte vector
- no host file or terminal dependencies in core APIs
- examples own printing, not library code

After Rust tests and examples are stable, add a thin WASM wrapper in a separate
step or demo repo.
