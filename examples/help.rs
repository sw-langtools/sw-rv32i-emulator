fn main() {
    println!("sw-rv32i-emulator examples");
    println!();
    println!("Run from sw-rv32i-emulator:");
    println!("  cargo run --example help");
    println!("  cargo run --example hello");
    println!("  cargo run --example byte_loaded");
    println!("  cargo run --example board_blink");
    println!("  cargo run --example board_uart_hello");
    println!(
        "  cargo run --example board_blink -- ../sw-rv32i-target/boards/esp32-c3-devkitm-1.toml"
    );
    println!(
        "  cargo run --example board_blink -- ../sw-rv32i-target/boards/ch32v003f4p6-evt.toml"
    );
    println!(
        "  cargo run --example board_uart_hello -- ../sw-rv32i-target/boards/esp32-c3-devkitm-1.toml"
    );
    println!();
    println!("Examples:");
    println!("  help");
    println!("    Prints this guide.");
    println!();
    println!("  hello");
    println!("    Assembles examples/asm/hello.s with the assembler crate, loads it into RAM,");
    println!("    executes it, and dumps bytes that spell hello from memory.");
    println!("    Current output path: RAM bytes, not UART.");
    println!("    Current default profile: RV32I.");
    println!();
    println!("  byte_loaded");
    println!("    Builds a program directly from Rust ISA instructions, loads bytes into RAM,");
    println!("    and demos arithmetic, branch, jump, load, store, and ebreak execution.");
    println!("    Use this when debugging emulator semantics without assembler syntax involved.");
    println!();
    println!("  board_blink");
    println!("    Loads board TOML files from sw-rv32i-target, derives the ISA profile from");
    println!("    the board arch field, assembles a shared source program with li/sw/lw");
    println!("    MMIO syntax, creates a target MmioBus, and runs it through Machine.");
    println!("    Default targets:");
    println!("      esp32-c3-devkitm-1       arch rv32imc -> emulator profile RV32IM");
    println!("      esp32-c5-devkitc-1       arch rv32imac -> emulator profile RV32IM");
    println!("      esp32-c6-devkitc-1       arch rv32imac -> emulator profile RV32IM");
    println!("      ch32v003f4p6-evt         arch rv32ec -> emulator profile RV32E");
    println!("    Current LED path: generic-gpio MMIO, not register-accurate vendor GPIO.");
    println!("    Current ESP32-C3 note: the board TOML uses GPIO8, but this demo treats it");
    println!("    as generic GPIO. Real ESP32-C3 DevKitM-1 RGB LED support needs WS2812/RMT");
    println!("    or bit-banged timing, not just GPIO high/low.");
    println!();
    println!("  board_uart_hello");
    println!("    Loads board TOML files, derives the profile from the board arch field,");
    println!("    assembles a shared source program with li/sw MMIO syntax, creates a");
    println!("    target MmioBus, and writes hello through uart0 generic-uart MMIO.");
    println!("    Default targets:");
    println!("      esp32-c3-devkitm-1       arch rv32imc -> emulator profile RV32IM");
    println!("      ch32v003f4p6-evt         arch rv32ec -> emulator profile RV32E");
    println!("    Current UART path: generic-uart MMIO, not register-accurate vendor UART.");
    println!();
    println!("Profiles:");
    println!("  Board-driven examples read the board TOML arch field:");
    println!("    rv32e* -> RV32E");
    println!("    arch containing m -> RV32IM");
    println!("    otherwise -> RV32I");
    println!("  Assembler APIs also expose assemble_with_profile(source, IsaProfile).");
    println!("  The assembler rejects RV32E-incompatible high registers when RV32E is used.");
    println!();
    println!("Targets:");
    println!("  Board metadata lives in ../sw-rv32i-target/boards/*.toml.");
    println!("  Pass one or more TOML paths after -- to board_blink to select targets.");
    println!("  Board TOML currently describes id, family, arch, RAM, GPIO pins, aliases,");
    println!("  generic-gpio MMIO, and generic-uart MMIO.");
    println!();
    println!("LED demos:");
    println!("  Runnable now:");
    println!(
        "    cargo run --example board_blink -- ../sw-rv32i-target/boards/esp32-c3-devkitm-1.toml"
    );
    println!(
        "    cargo run --example board_blink -- ../sw-rv32i-target/boards/ch32v003f4p6-evt.toml"
    );
    println!("  These are emulator generic-GPIO demos. They verify target TOML/profile/MMIO");
    println!("  routing and GPIO traces, not vendor-register-accurate electrical behavior.");
    println!();
    println!("Text and UART demos:");
    println!("  Runnable now:");
    println!("    cargo run --example hello");
    println!("    cargo run --example board_uart_hello");
    println!(
        "    cargo run --example board_uart_hello -- ../sw-rv32i-target/boards/ch32v003f4p6-evt.toml"
    );
    println!("  hello writes to RAM; board_uart_hello writes through board TOML uart0.");
    println!();
    println!("I2C and display demos:");
    println!("  Not implemented yet.");
    println!("  Planned shape:");
    println!("    board TOML declares generic-i2c or board-specific I2C/SPI display device");
    println!("    emulator routes MMIO transactions to the device model");
    println!("    shared demos bind to logical aliases like display0 rather than board ids");
    println!();
    println!("Assembler-source MMIO status:");
    println!("  Runnable now for LED blink and UART hello.");
    println!("  The assembler supports li for full 32-bit constants plus lw/sw for word MMIO.");
}
