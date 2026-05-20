use std::env;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use sw_rv32i_emulator::{Machine, fetch_decode, format_cpu_state, step_machine_with_profile};
use sw_rv32i_isa::{IsaProfile, disassemble};
use sw_rv32i_target::{Board, MmioBus, load_board_file};

const DEFAULT_BOARDS: &[&str] = &[
    "../sw-rv32i-target/boards/esp32-c3-devkitm-1.toml",
    "../sw-rv32i-target/boards/ch32v003f4p6-evt.toml",
];
const MESSAGE: &[u8] = b"hello\n";

fn main() {
    if let Err(err) = run_demo() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_demo() -> Result<(), String> {
    let board_paths: Vec<PathBuf> = env::args_os().skip(1).map(PathBuf::from).collect();
    let board_paths: Vec<PathBuf> = if board_paths.is_empty() {
        DEFAULT_BOARDS.iter().map(PathBuf::from).collect()
    } else {
        board_paths
    };

    println!("== Board UART Hello Demo ==");
    println!("program: load board uart0 base, write hello bytes to generic UART TX, halt");

    for path in board_paths {
        run_board(&path)?;
    }

    Ok(())
}

fn run_board(path: &Path) -> Result<(), String> {
    let board = load_board_file(path).map_err(|err| format!("{path:?}: {err}"))?;
    let uart_base = board
        .mmio_base("uart0")
        .ok_or_else(|| format!("{} has no uart0 MMIO device", board.id))?;
    let profile = profile_for_board(&board);
    let source = uart_hello_source(uart_base);
    let program = sw_rv32i_asm::assemble_with_profile(&source, profile).map_err(|err| {
        format!(
            "{} board_uart_hello source failed profile validation: {err}",
            board.id
        )
    })?;

    println!("\n== Board: {} ==", board.id);
    println!("path: {}", path.display());
    println!("family: {}", board.family);
    println!("arch: {}", board.arch);
    println!("uart0 mmio: 0x{uart_base:08x}");

    println!("\nSource:");
    print!("{source}");

    println!("\nProgram Bytes:");
    print!("{}", format_bytes(&program));

    let mmio = MmioBus::for_board(&board).map_err(|err| err.to_string())?;
    let mut machine = Machine::new(256, mmio);
    machine
        .memory
        .load(0, &program)
        .map_err(|err| format!("{err:?}"))?;

    println!("\nExecution Trace:");
    while !machine.cpu.halted() {
        let pc = machine.cpu.pc();
        let insn = fetch_decode(&machine.cpu, &machine.memory).map_err(|err| format!("{err:?}"))?;
        let mut text = String::new();
        disassemble(insn, &mut text).map_err(|err| err.to_string())?;
        step_machine_with_profile(&mut machine, profile).map_err(|err| format!("{err:?}"))?;
        println!("{pc:04x}: {text}");
    }

    println!("\nUART Output:");
    print!(
        "{}",
        machine
            .mmio
            .uart_output_string("uart0")
            .map_err(|err| err.to_string())?
    );

    println!("\nFinal CPU State:");
    print!("{}", format_cpu_state(&machine.cpu));

    Ok(())
}

fn profile_for_board(board: &Board) -> IsaProfile {
    if board.arch.starts_with("rv32e") {
        IsaProfile::RV32E
    } else if board.arch.contains('m') {
        IsaProfile::RV32IM
    } else {
        IsaProfile::RV32I
    }
}

fn uart_hello_source(uart_base: u32) -> String {
    let mut source = format!("li x1, 0x{uart_base:08x}\n");
    for byte in MESSAGE {
        use std::fmt::Write as _;
        writeln!(&mut source, "li x2, {}", *byte).unwrap();
        source.push_str("sw x2, 0(x1)\n");
    }
    source.push_str("ebreak\n");
    source
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
