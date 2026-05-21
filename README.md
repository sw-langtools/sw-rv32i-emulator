# sw-rv32i-emulator

RV32I emulator.

## Status

This crate currently contains the hello-world MVP path plus incremental board
emulation demos:

- assemble source through `sw-rv32i-asm`
- load bytes into memory or a board-aware `Machine`
- execute RV32I instructions needed by the current examples
- verify memory output for `hello\n`
- route board MMIO word loads/stores through `sw-rv32i-target::MmioBus`

The broader RV32I emulator is intentionally incremental. Full CPU state,
complete memory helpers, all RV32I execution semantics, dump helpers, and richer
examples remain queued in follow-up agentrail steps.

## Examples

```bash
cargo run --example hello
cargo run --example byte_loaded
cargo run --example board_blink
cargo run --example board_uart_hello
cargo run --example help
```

`board_blink` defaults to the ESP32-C3/C5/C6 placeholder board TOML files in
`sw-rv32i-target` plus the CH32V003 RV32E placeholder, assembles the same blink
source program for each board's `led` alias, derives the emulator profile from
the board `arch`, and prints the GPIO MMIO trace.

`board_uart_hello` defaults to ESP32-C3 plus CH32V003, assembles one hello
source program for each board's `uart0` generic UART MMIO, derives the emulator
profile from the board `arch`, and prints the captured UART output.

## Sibling Layout

Cross-crate deps assume sibling clones at `~/github/sw-langtools/`:

```
sw-langtools/
  sw-rv32i-isa/
  sw-rv32i-asm/
  sw-rv32i-emulator/
  sw-rv32i-target/
```

## License

MIT.
