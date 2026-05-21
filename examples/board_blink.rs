use std::env;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use sw_rv32i_emulator::{Machine, fetch_decode, format_cpu_state, step_machine_with_profile};
use sw_rv32i_isa::{IsaProfile, disassemble};
use sw_rv32i_target::{
    Board, GenericGpioMmio, MmioBus, generic_gpio_led_blink_binding, load_board_file,
};

const DEFAULT_BOARDS: &[&str] = &[
    "../sw-rv32i-target/boards/esp32-c3-devkitm-1.toml",
    "../sw-rv32i-target/boards/esp32-c5-devkitc-1.toml",
    "../sw-rv32i-target/boards/esp32-c6-devkitc-1.toml",
    "../sw-rv32i-target/boards/ch32v003f4p6-evt.toml",
];

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

    println!("== Shared Board Blink Demo ==");
    println!(
        "program: load board gpio base, load board led mask, set led, read state, clear led, halt"
    );

    for path in board_paths {
        run_board(&path)?;
    }

    Ok(())
}

fn run_board(path: &Path) -> Result<(), String> {
    let board = load_board_file(path).map_err(|err| format!("{path:?}: {err}"))?;
    let binding = generic_gpio_led_blink_binding(&board)
        .map_err(|err| format!("{} cannot run board_blink: {err}", board.id))?;
    let profile = profile_for_board(&board);
    let source = blink_source(binding.gpio_base, binding.led_mask);
    let program = sw_rv32i_asm::assemble_with_profile(&source, profile).map_err(|err| {
        format!(
            "{} board_blink source failed profile validation: {err}",
            board.id
        )
    })?;

    println!("\n== Board: {} ==", board.id);
    println!("path: {}", path.display());
    println!("family: {}", board.family);
    println!("arch: {}", board.arch);
    println!(
        "led: gpio{} mask 0x{:08x}",
        binding.led_pin, binding.led_mask
    );
    println!("gpio mmio: 0x{:08x}", binding.gpio_base);

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

    println!("\nGPIO Trace:");
    let trace = machine
        .mmio
        .gpio()
        .ok_or_else(|| format!("{} has no generic GPIO MMIO state", board.id))?
        .trace();
    for (index, event) in trace.iter().enumerate() {
        let level = if event.high { "high" } else { "low" };
        println!("{index}: gpio{} {level}", event.pin);
    }

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

fn blink_source(gpio_base: u32, led_mask: u32) -> String {
    format!(
        "li x1, 0x{gpio_base:08x}\n\
         li x2, {led_mask}\n\
         sw x2, {}(x1)\n\
         lw x3, {}(x1)\n\
         sw x2, {}(x1)\n\
         ebreak\n",
        GenericGpioMmio::SET_OFFSET,
        GenericGpioMmio::READ_OFFSET,
        GenericGpioMmio::CLEAR_OFFSET,
    )
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
