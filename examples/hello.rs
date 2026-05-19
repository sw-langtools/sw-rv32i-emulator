use std::fmt::Write;

use sw_rv32i_emulator::{CpuState, Memory, fetch_decode, step};
use sw_rv32i_isa::{Reg, disassemble};

const SOURCE: &str = include_str!("asm/hello.s");
const OUTPUT_ADDR: usize = 0x100;
const OUTPUT_LEN: usize = 6;

fn main() {
    if let Err(err) = run_demo() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_demo() -> Result<(), String> {
    println!("== Source: examples/asm/hello.s ==");
    print!("{SOURCE}");

    let program = sw_rv32i_asm::assemble(SOURCE).map_err(|err| err.to_string())?;

    println!("\n== Assembled Bytes ==");
    println!("{}", format_bytes(&program));

    println!("\n== Intermediate Decode/Disassembly ==");
    for (index, chunk) in program.chunks_exact(4).enumerate() {
        let pc = index * 4;
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        let insn = sw_rv32i_isa::decode_word(word).map_err(|err| format!("{err:?}"))?;
        let mut text = String::new();
        disassemble(insn, &mut text).map_err(|err| err.to_string())?;
        println!("{pc:04x}: {word:08x}  {text}");
    }

    let mut cpu = CpuState::new();
    let mut memory = Memory::new(512);
    memory.load(0, &program).map_err(|err| format!("{err:?}"))?;

    println!("\n== Load ==");
    println!("loaded {} bytes at 0x0000", program.len());

    println!("\n== Execution Trace ==");
    while !cpu.halted() {
        let pc = cpu.pc();
        let insn = fetch_decode(&cpu, &memory).map_err(|err| format!("{err:?}"))?;
        let mut text = String::new();
        disassemble(insn, &mut text).map_err(|err| err.to_string())?;
        step(&mut cpu, &mut memory).map_err(|err| format!("{err:?}"))?;
        println!("{pc:04x}: {text}");
    }

    println!("\n== Final CPU State ==");
    print!("{}", format_cpu_state(&cpu));

    println!("\n== Final Memory Dump ==");
    print!("{}", format_memory_dump(&memory, OUTPUT_ADDR, OUTPUT_LEN));

    println!("\n== Emulator Output ==");
    println!(
        "{}",
        std::str::from_utf8(memory.bytes(OUTPUT_ADDR..OUTPUT_ADDR + OUTPUT_LEN))
            .map_err(|err| err.to_string())?
    );

    Ok(())
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

fn format_cpu_state(cpu: &CpuState) -> String {
    let mut out = String::new();
    writeln!(&mut out, "pc: 0x{:08x}", cpu.pc()).unwrap();
    writeln!(&mut out, "halted: {}", cpu.halted()).unwrap();
    writeln!(&mut out, "instr_count: {}", cpu.instr_count()).unwrap();
    for reg in [Reg::X1, Reg::X5] {
        writeln!(
            &mut out,
            "{} ({}): 0x{:08x}",
            reg,
            reg.abi_name(),
            cpu.read_reg(reg)
        )
        .unwrap();
    }
    out
}

fn format_memory_dump(memory: &Memory, start: usize, len: usize) -> String {
    let mut out = String::new();
    let bytes = memory.bytes(start..start + len);
    write!(&mut out, "{start:04x}:").unwrap();
    for byte in bytes {
        write!(&mut out, " {byte:02x}").unwrap();
    }
    write!(&mut out, "  |").unwrap();
    for byte in bytes {
        let ch = match *byte {
            b'\n' => '.',
            0x20..=0x7e => *byte as char,
            _ => '.',
        };
        out.push(ch);
    }
    out.push_str("|\n");
    out
}
