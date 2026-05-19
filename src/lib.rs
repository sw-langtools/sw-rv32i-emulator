//! Minimal RV32I emulator for the hello-world MVP.

use sw_rv32i_isa::{ImmOp, Instruction, Reg, StoreWidth};

pub struct CpuState {
    regs: [u32; 32],
    pc: u32,
    halted: bool,
}

impl CpuState {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            halted: false,
        }
    }

    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn halted(&self) -> bool {
        self.halted
    }

    pub fn read_reg(&self, reg: Reg) -> u32 {
        self.regs[reg.index_u8() as usize]
    }

    pub fn write_reg(&mut self, reg: Reg, value: u32) {
        if reg != Reg::X0 {
            self.regs[reg.index_u8() as usize] = value;
        }
    }
}

impl Default for CpuState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Memory {
    bytes: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
        }
    }

    pub fn load(&mut self, addr: u32, bytes: &[u8]) -> Result<(), ExecError> {
        let start = addr as usize;
        let end = start
            .checked_add(bytes.len())
            .ok_or(ExecError::MemoryOutOfBounds)?;
        let dst = self
            .bytes
            .get_mut(start..end)
            .ok_or(ExecError::MemoryOutOfBounds)?;
        dst.copy_from_slice(bytes);
        Ok(())
    }

    pub fn read_u32(&self, addr: u32) -> Result<u32, ExecError> {
        if addr & 0x3 != 0 {
            return Err(ExecError::MisalignedFetch);
        }
        let start = addr as usize;
        let bytes = self
            .bytes
            .get(start..start + 4)
            .ok_or(ExecError::MemoryOutOfBounds)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn write_u8(&mut self, addr: u32, value: u8) -> Result<(), ExecError> {
        *self
            .bytes
            .get_mut(addr as usize)
            .ok_or(ExecError::MemoryOutOfBounds)? = value;
        Ok(())
    }

    pub fn bytes(&self, range: core::ops::Range<usize>) -> &[u8] {
        &self.bytes[range]
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExecError {
    Halted,
    Decode,
    MemoryOutOfBounds,
    MisalignedFetch,
    UnsupportedInstruction,
}

pub fn step(cpu: &mut CpuState, mem: &mut Memory) -> Result<(), ExecError> {
    if cpu.halted {
        return Err(ExecError::Halted);
    }

    let word = mem.read_u32(cpu.pc)?;
    let insn = sw_rv32i_isa::decode_word(word).map_err(|_| ExecError::Decode)?;
    cpu.pc = cpu.pc.wrapping_add(4);

    match insn {
        Instruction::OpImm {
            op: ImmOp::Addi,
            rd,
            rs1,
            imm,
        } => {
            let value = cpu.read_reg(rs1).wrapping_add(imm as u32);
            cpu.write_reg(rd, value);
            Ok(())
        }
        Instruction::Store {
            width: StoreWidth::Byte,
            rs1,
            rs2,
            offset,
        } => {
            let addr = cpu.read_reg(rs1).wrapping_add(offset as u32);
            mem.write_u8(addr, cpu.read_reg(rs2) as u8)
        }
        Instruction::Ebreak => {
            cpu.halted = true;
            Ok(())
        }
        _ => Err(ExecError::UnsupportedInstruction),
    }
}

pub fn run(cpu: &mut CpuState, mem: &mut Memory, max_steps: usize) -> Result<usize, ExecError> {
    let mut steps = 0;
    while !cpu.halted && steps < max_steps {
        step(cpu, mem)?;
        steps += 1;
    }
    Ok(steps)
}

pub fn assemble_load_run(
    source: &str,
    mem_size: usize,
    max_steps: usize,
) -> Result<(CpuState, Memory, usize), MvpError> {
    let program = sw_rv32i_asm::assemble(source)?;
    let mut cpu = CpuState::new();
    let mut mem = Memory::new(mem_size);
    mem.load(0, &program)?;
    let steps = run(&mut cpu, &mut mem, max_steps)?;
    Ok((cpu, mem, steps))
}

#[derive(Debug)]
pub enum MvpError {
    Asm(sw_rv32i_asm::AsmError),
    Exec(ExecError),
}

impl From<sw_rv32i_asm::AsmError> for MvpError {
    fn from(value: sw_rv32i_asm::AsmError) -> Self {
        MvpError::Asm(value)
    }
}

impl From<ExecError> for MvpError {
    fn from(value: ExecError) -> Self {
        MvpError::Exec(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO_SOURCE: &str = r#"
        addi x1, x0, 0x100
        addi t0, zero, 0x68
        sb t0, 0(x1)
        addi t0, zero, 0x65
        sb t0, 1(x1)
        addi t0, zero, 0x6c
        sb t0, 2(x1)
        sb t0, 3(x1)
        addi t0, zero, 0x6f
        sb t0, 4(x1)
        addi t0, zero, 0x0a
        sb t0, 5(x1)
        ebreak
    "#;

    #[test]
    fn x0_writes_are_ignored() {
        let mut cpu = CpuState::new();
        cpu.write_reg(Reg::X0, 123);
        assert_eq!(cpu.read_reg(Reg::X0), 0);
    }

    #[test]
    fn runs_assembled_hello_world_to_memory() {
        let (cpu, mem, steps) = assemble_load_run(HELLO_SOURCE, 512, 100).unwrap();
        assert!(cpu.halted());
        assert_eq!(steps, 13);
        assert_eq!(mem.bytes(0x100..0x106), b"hello\n");
    }

    #[test]
    fn rejects_non_mvp_instruction() {
        let program = sw_rv32i_isa::encode_word(Instruction::Ecall)
            .unwrap()
            .to_le_bytes();
        let mut cpu = CpuState::new();
        let mut mem = Memory::new(64);
        mem.load(0, &program).unwrap();
        assert_eq!(
            step(&mut cpu, &mut mem),
            Err(ExecError::UnsupportedInstruction)
        );
    }
}
