# sw-rv32i-emulator

RV32I emulator.

## Status

This crate currently contains the hello-world MVP path:

- assemble source through `sw-rv32i-asm`
- load bytes into memory
- execute the minimal subset needed for the MVP: `addi`, `sb`, and `ebreak`
- verify memory output for `hello\n`

The broader RV32I emulator is intentionally incremental. Full CPU state,
complete memory helpers, all RV32I execution semantics, dump helpers, and richer
examples remain queued in follow-up agentrail steps.

## Sibling Layout

Cross-crate deps assume sibling clones at `~/github/sw-langtools/`:

```
sw-langtools/
  sw-rv32i-isa/
  sw-rv32i-asm/
  sw-rv32i-emulator/
```

## License

MIT.
