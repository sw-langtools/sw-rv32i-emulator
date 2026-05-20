use std::fmt::Write;

use sw_rv32i_emulator::{CpuState, Memory, fetch_decode, format_cpu_state, format_hex_dump, step};
use sw_rv32i_isa::disassemble;

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
    print!(
        "{}",
        format_hex_dump(&memory, OUTPUT_ADDR, OUTPUT_LEN).map_err(|err| format!("{err:?}"))?
    );

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
