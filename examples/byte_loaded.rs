use std::fmt::Write;

use sw_rv32i_emulator::{CpuState, Memory, format_cpu_state, format_hex_dump, run};
use sw_rv32i_isa::{BranchCond, ImmOp, Instruction, LoadWidth, Reg, RegOp, StoreWidth};

const DATA_ADDR: usize = 0x80;

fn main() {
    if let Err(err) = run_demo() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_demo() -> Result<(), String> {
    println!("== Source-Equivalent Comments ==");
    println!("addi x1, x0, 0x80      # data base");
    println!("addi x2, x0, 7         # lhs");
    println!("addi x3, x0, 5         # rhs");
    println!("add  x4, x2, x3        # arithmetic");
    println!("sw   x4, 0(x1)         # store result");
    println!("lw   x5, 0(x1)         # load result");
    println!("beq  x4, x5, +12       # jump to success marker");
    println!("addi x6, x0, 1         # skipped");
    println!("jal  x0, +8            # skip success marker");
    println!("addi x6, x0, 2         # success marker");
    println!("ebreak");

    let program = encode_program(&[
        Instruction::OpImm {
            op: ImmOp::Addi,
            rd: Reg::X1,
            rs1: Reg::X0,
            imm: DATA_ADDR as i32,
        },
        Instruction::OpImm {
            op: ImmOp::Addi,
            rd: Reg::X2,
            rs1: Reg::X0,
            imm: 7,
        },
        Instruction::OpImm {
            op: ImmOp::Addi,
            rd: Reg::X3,
            rs1: Reg::X0,
            imm: 5,
        },
        Instruction::Op {
            op: RegOp::Add,
            rd: Reg::X4,
            rs1: Reg::X2,
            rs2: Reg::X3,
        },
        Instruction::Store {
            width: StoreWidth::Word,
            rs1: Reg::X1,
            rs2: Reg::X4,
            offset: 0,
        },
        Instruction::Load {
            width: LoadWidth::Word,
            rd: Reg::X5,
            rs1: Reg::X1,
            offset: 0,
        },
        Instruction::Branch {
            cond: BranchCond::Eq,
            rs1: Reg::X4,
            rs2: Reg::X5,
            offset: 12,
        },
        Instruction::OpImm {
            op: ImmOp::Addi,
            rd: Reg::X6,
            rs1: Reg::X0,
            imm: 1,
        },
        Instruction::Jal {
            rd: Reg::X0,
            offset: 8,
        },
        Instruction::OpImm {
            op: ImmOp::Addi,
            rd: Reg::X6,
            rs1: Reg::X0,
            imm: 2,
        },
        Instruction::Ebreak,
    ])?;

    println!("\n== Program Bytes ==");
    print!("{}", format_bytes(&program));

    let mut cpu = CpuState::new();
    let mut memory = Memory::new(256);
    memory.load(0, &program).map_err(|err| format!("{err:?}"))?;
    let steps = run(&mut cpu, &mut memory, 100).map_err(|err| format!("{err:?}"))?;

    println!("\n== Run ==");
    println!("steps: {steps}");

    println!("\n== Final CPU State ==");
    print!("{}", format_cpu_state(&cpu));

    println!("\n== Data Memory ==");
    print!(
        "{}",
        format_hex_dump(&memory, DATA_ADDR, 4).map_err(|err| format!("{err:?}"))?
    );

    Ok(())
}

fn encode_program(insns: &[Instruction]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(insns.len() * 4);
    for insn in insns.iter().copied() {
        let word = sw_rv32i_isa::encode_word(insn).map_err(|err| format!("{err:?}"))?;
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    Ok(bytes)
}

fn format_bytes(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (offset, chunk) in bytes.chunks(16).enumerate() {
        write!(&mut out, "{:04x}:", offset * 16).unwrap();
        for byte in chunk {
            write!(&mut out, " {byte:02x}").unwrap();
        }
        out.push('\n');
    }
    out
}
