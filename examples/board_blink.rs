use std::env;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use sw_rv32i_emulator::{Machine, fetch_decode, format_cpu_state, step_machine_with_profile};
use sw_rv32i_isa::{ImmOp, Instruction, IsaProfile, LoadWidth, Reg, StoreWidth, disassemble};
use sw_rv32i_target::{Board, GenericGpioMmio, MmioBus, load_board_file};

const DEFAULT_BOARDS: &[&str] = &[
    "../sw-rv32i-target/boards/esp32-c3-devkitm-1.toml",
    "../sw-rv32i-target/boards/esp32-c5-devkitc-1.toml",
    "../sw-rv32i-target/boards/esp32-c6-devkitc-1.toml",
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
    let gpio_base = board
        .mmio_base("gpio")
        .ok_or_else(|| format!("{} has no gpio MMIO device", board.id))?;
    let led_pin = board
        .led_gpio()
        .ok_or_else(|| format!("{} has no led alias", board.id))?;
    let led_mask = 1u32
        .checked_shl(led_pin)
        .ok_or_else(|| format!("{} led pin {led_pin} cannot fit in a GPIO mask", board.id))?;
    let program = blink_program(gpio_base, led_mask)?;

    println!("\n== Board: {} ==", board.id);
    println!("path: {}", path.display());
    println!("family: {}", board.family);
    println!("arch: {}", board.arch);
    println!("led: gpio{led_pin} mask 0x{led_mask:08x}");
    println!("gpio mmio: 0x{gpio_base:08x}");

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
        step_machine_with_profile(&mut machine, profile_for_board(&board))
            .map_err(|err| format!("{err:?}"))?;
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

fn blink_program(gpio_base: u32, led_mask: u32) -> Result<Vec<u8>, String> {
    let mut insns = Vec::new();
    insns.extend(load_const(Reg::X1, gpio_base));
    insns.extend(load_const(Reg::X2, led_mask));
    insns.extend([
        Instruction::Store {
            width: StoreWidth::Word,
            rs1: Reg::X1,
            rs2: Reg::X2,
            offset: GenericGpioMmio::SET_OFFSET as i32,
        },
        Instruction::Load {
            width: LoadWidth::Word,
            rd: Reg::X3,
            rs1: Reg::X1,
            offset: GenericGpioMmio::READ_OFFSET as i32,
        },
        Instruction::Store {
            width: StoreWidth::Word,
            rs1: Reg::X1,
            rs2: Reg::X2,
            offset: GenericGpioMmio::CLEAR_OFFSET as i32,
        },
        Instruction::Ebreak,
    ]);

    let mut bytes = Vec::with_capacity(insns.len() * 4);
    for insn in insns {
        let word = sw_rv32i_isa::encode_word(insn).map_err(|err| format!("{err:?}"))?;
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    Ok(bytes)
}

fn load_const(rd: Reg, value: u32) -> [Instruction; 2] {
    let hi = value.wrapping_add(0x800) & 0xffff_f000;
    let lo = value.wrapping_sub(hi) as i32;
    [
        Instruction::Lui { rd, imm: hi as i32 },
        Instruction::OpImm {
            op: ImmOp::Addi,
            rd,
            rs1: rd,
            imm: lo,
        },
    ]
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
