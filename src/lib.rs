//! Minimal RV32I emulator for the hello-world MVP.

use sw_rv32i_isa::{ImmOp, Instruction, Reg, StoreWidth};

pub struct CpuState {
    regs: [u32; 32],
    pc: u32,
    halted: bool,
    instr_count: u64,
}

impl CpuState {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            halted: false,
            instr_count: 0,
        }
    }

    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn halted(&self) -> bool {
        self.halted
    }

    pub fn set_halted(&mut self, halted: bool) {
        self.halted = halted;
    }

    pub fn instr_count(&self) -> u64 {
        self.instr_count
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    pub fn advance_pc(&mut self, bytes: u32) {
        self.pc = self.pc.wrapping_add(bytes);
    }

    pub fn read_reg(&self, reg: Reg) -> u32 {
        self.regs[reg.index_u8() as usize]
    }

    pub fn write_reg(&mut self, reg: Reg, value: u32) {
        if reg != Reg::X0 {
            self.regs[reg.index_u8() as usize] = value;
        }
    }

    pub fn regs(&self) -> &[u32; 32] {
        &self.regs
    }

    pub fn increment_instr_count(&mut self) {
        self.instr_count = self.instr_count.wrapping_add(1);
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

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn read_u8(&self, addr: u32) -> Result<u8, ExecError> {
        self.bytes
            .get(addr as usize)
            .copied()
            .ok_or(ExecError::MemoryOutOfBounds)
    }

    pub fn read_u16(&self, addr: u32) -> Result<u16, ExecError> {
        if addr & 0x1 != 0 {
            return Err(ExecError::MisalignedDataAccess);
        }
        let start = addr as usize;
        let bytes = self
            .bytes
            .get(start..start + 2)
            .ok_or(ExecError::MemoryOutOfBounds)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_u32(&self, addr: u32) -> Result<u32, ExecError> {
        self.read_aligned_u32(addr, ExecError::MisalignedDataAccess)
    }

    pub fn fetch_u32(&self, addr: u32) -> Result<u32, ExecError> {
        self.read_aligned_u32(addr, ExecError::MisalignedFetch)
    }

    pub fn write_u8(&mut self, addr: u32, value: u8) -> Result<(), ExecError> {
        *self
            .bytes
            .get_mut(addr as usize)
            .ok_or(ExecError::MemoryOutOfBounds)? = value;
        Ok(())
    }

    pub fn write_u16(&mut self, addr: u32, value: u16) -> Result<(), ExecError> {
        if addr & 0x1 != 0 {
            return Err(ExecError::MisalignedDataAccess);
        }
        let start = addr as usize;
        let dst = self
            .bytes
            .get_mut(start..start + 2)
            .ok_or(ExecError::MemoryOutOfBounds)?;
        dst.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn write_u32(&mut self, addr: u32, value: u32) -> Result<(), ExecError> {
        if addr & 0x3 != 0 {
            return Err(ExecError::MisalignedDataAccess);
        }
        let start = addr as usize;
        let dst = self
            .bytes
            .get_mut(start..start + 4)
            .ok_or(ExecError::MemoryOutOfBounds)?;
        dst.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn read_aligned_u32(&self, addr: u32, align_error: ExecError) -> Result<u32, ExecError> {
        if addr & 0x3 != 0 {
            return Err(align_error);
        }
        let start = addr as usize;
        let bytes = self
            .bytes
            .get(start..start + 4)
            .ok_or(ExecError::MemoryOutOfBounds)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
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
    MisalignedDataAccess,
    UnsupportedInstruction,
    RunLimitReached,
}

pub fn fetch_decode(cpu: &CpuState, mem: &Memory) -> Result<Instruction, ExecError> {
    let word = mem.fetch_u32(cpu.pc())?;
    sw_rv32i_isa::decode_word(word).map_err(|_| ExecError::Decode)
}

pub fn step(cpu: &mut CpuState, mem: &mut Memory) -> Result<(), ExecError> {
    if cpu.halted {
        return Err(ExecError::Halted);
    }

    let insn = fetch_decode(cpu, mem)?;
    cpu.advance_pc(4);

    let result = match insn {
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
            cpu.set_halted(true);
            Ok(())
        }
        _ => Err(ExecError::UnsupportedInstruction),
    };
    if result.is_ok() {
        cpu.increment_instr_count();
    }
    result
}

pub fn run(cpu: &mut CpuState, mem: &mut Memory, max_steps: usize) -> Result<usize, ExecError> {
    let mut steps = 0;
    while !cpu.halted && steps < max_steps {
        step(cpu, mem)?;
        steps += 1;
    }
    if !cpu.halted {
        return Err(ExecError::RunLimitReached);
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
        assert_eq!(cpu.instr_count(), 13);
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

    #[test]
    fn cpu_state_tracks_pc_registers_halt_and_instruction_count() {
        let mut cpu = CpuState::new();
        cpu.set_pc(0xffff_fffc);
        cpu.advance_pc(8);
        assert_eq!(cpu.pc(), 4);

        cpu.write_reg(Reg::X1, 42);
        cpu.write_reg(Reg::X0, 99);
        assert_eq!(cpu.read_reg(Reg::X1), 42);
        assert_eq!(cpu.read_reg(Reg::X0), 0);
        assert_eq!(cpu.regs()[1], 42);

        cpu.increment_instr_count();
        cpu.set_halted(true);
        assert_eq!(cpu.instr_count(), 1);
        assert!(cpu.halted());
    }

    #[test]
    fn memory_reads_and_writes_little_endian_values() {
        let mut mem = Memory::new(16);
        assert_eq!(mem.len(), 16);

        mem.write_u8(0, 0xaa).unwrap();
        mem.write_u16(2, 0x1234).unwrap();
        mem.write_u32(4, 0x89ab_cdef).unwrap();

        assert_eq!(mem.read_u8(0), Ok(0xaa));
        assert_eq!(mem.read_u16(2), Ok(0x1234));
        assert_eq!(mem.read_u32(4), Ok(0x89ab_cdef));
        assert_eq!(
            mem.bytes(0..8),
            &[0xaa, 0, 0x34, 0x12, 0xef, 0xcd, 0xab, 0x89]
        );
    }

    #[test]
    fn memory_reports_bounds_and_alignment_errors() {
        let mut mem = Memory::new(4);

        assert_eq!(mem.load(3, &[1, 2]), Err(ExecError::MemoryOutOfBounds));
        assert_eq!(mem.read_u8(4), Err(ExecError::MemoryOutOfBounds));
        assert_eq!(mem.write_u8(4, 1), Err(ExecError::MemoryOutOfBounds));
        assert_eq!(mem.read_u16(1), Err(ExecError::MisalignedDataAccess));
        assert_eq!(mem.write_u16(1, 1), Err(ExecError::MisalignedDataAccess));
        assert_eq!(mem.read_u32(2), Err(ExecError::MisalignedDataAccess));
        assert_eq!(mem.write_u32(2, 1), Err(ExecError::MisalignedDataAccess));
        assert_eq!(mem.fetch_u32(2), Err(ExecError::MisalignedFetch));
    }

    #[test]
    fn fetch_decode_reports_alignment_and_decode_errors() {
        let mut cpu = CpuState::new();
        let mut mem = Memory::new(8);

        cpu.set_pc(2);
        assert_eq!(fetch_decode(&cpu, &mem), Err(ExecError::MisalignedFetch));

        cpu.set_pc(0);
        mem.write_u32(0, 0).unwrap();
        assert_eq!(fetch_decode(&cpu, &mem), Err(ExecError::Decode));
    }

    #[test]
    fn step_advances_pc_and_counts_only_successful_instructions() {
        let program = sw_rv32i_asm::assemble(
            r#"
            addi x1, x0, 1
            ebreak
            "#,
        )
        .unwrap();
        let mut cpu = CpuState::new();
        let mut mem = Memory::new(32);
        mem.load(0, &program).unwrap();

        step(&mut cpu, &mut mem).unwrap();
        assert_eq!(cpu.pc(), 4);
        assert_eq!(cpu.instr_count(), 1);
        assert_eq!(cpu.read_reg(Reg::X1), 1);

        step(&mut cpu, &mut mem).unwrap();
        assert_eq!(cpu.pc(), 8);
        assert_eq!(cpu.instr_count(), 2);
        assert!(cpu.halted());
        assert_eq!(step(&mut cpu, &mut mem), Err(ExecError::Halted));
        assert_eq!(cpu.instr_count(), 2);
    }

    #[test]
    fn run_reports_limit_when_cpu_does_not_halt() {
        let program = sw_rv32i_asm::assemble("addi x1, x0, 1").unwrap();
        let mut cpu = CpuState::new();
        let mut mem = Memory::new(32);
        mem.load(0, &program).unwrap();

        assert_eq!(run(&mut cpu, &mut mem, 1), Err(ExecError::RunLimitReached));
        assert_eq!(cpu.pc(), 4);
        assert_eq!(cpu.instr_count(), 1);
        assert!(!cpu.halted());
    }
}
