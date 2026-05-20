use sw_rv32i_emulator::{
    CpuState, Memory, assemble_load_run, fetch_decode, format_cpu_state, format_hex_dump, step,
};
use sw_rv32i_isa::disassemble;

const HELLO_SOURCE: &str = include_str!("../examples/asm/hello.s");
const OUTPUT_ADDR: usize = 0x100;
const OUTPUT_LEN: usize = 6;

#[test]
fn assembled_hello_source_runs_end_to_end() {
    let (cpu, memory, steps) = assemble_load_run(HELLO_SOURCE, 512, 100).unwrap();

    assert_eq!(steps, 13);
    assert!(cpu.halted());
    assert_eq!(cpu.pc(), 0x34);
    assert_eq!(cpu.instr_count(), 13);
    assert_eq!(
        memory.bytes(OUTPUT_ADDR..OUTPUT_ADDR + OUTPUT_LEN),
        b"hello\n"
    );
}

#[test]
fn assembled_hello_listing_and_dumps_are_stable() {
    let program = sw_rv32i_asm::assemble(HELLO_SOURCE).unwrap();
    let listing = instruction_listing(&program);

    assert!(listing.contains("0000: 10000093  addi x1, x0, 256\n"));
    assert!(listing.contains("0030: 00100073  ebreak\n"));

    let (cpu, memory, _) = assemble_load_run(HELLO_SOURCE, 512, 100).unwrap();
    assert_eq!(
        format_cpu_state(&cpu),
        "pc: 0x00000034\nhalted: true\ninstr_count: 13\nx1 (ra): 0x00000100\nx5 (t0): 0x0000000a\n"
    );
    assert_eq!(
        format_hex_dump(&memory, OUTPUT_ADDR, OUTPUT_LEN).unwrap(),
        "0100: 68 65 6c 6c 6f 0a                                |hello.|\n"
    );
}

#[test]
fn assembled_hello_can_be_traced_instruction_by_instruction() {
    let program = sw_rv32i_asm::assemble(HELLO_SOURCE).unwrap();
    let mut cpu = CpuState::new();
    let mut memory = Memory::new(512);
    memory.load(0, &program).unwrap();

    let mut trace = Vec::new();
    while !cpu.halted() {
        let pc = cpu.pc();
        let insn = fetch_decode(&cpu, &memory).unwrap();
        let mut text = String::new();
        disassemble(insn, &mut text).unwrap();
        trace.push(format!("{pc:04x}: {text}"));
        step(&mut cpu, &mut memory).unwrap();
    }

    assert_eq!(
        trace.first().map(String::as_str),
        Some("0000: addi x1, x0, 256")
    );
    assert_eq!(trace.last().map(String::as_str), Some("0030: ebreak"));
    assert_eq!(trace.len(), 13);
}

fn instruction_listing(program: &[u8]) -> String {
    let mut listing = String::new();
    for (index, chunk) in program.chunks_exact(4).enumerate() {
        let pc = index * 4;
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        let insn = sw_rv32i_isa::decode_word(word).unwrap();
        let mut text = String::new();
        disassemble(insn, &mut text).unwrap();
        listing.push_str(&format!("{pc:04x}: {word:08x}  {text}\n"));
    }
    listing
}
