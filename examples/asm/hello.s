# Write "hello\n" to memory at 0x100, then halt with ebreak.
addi x1, x0, 0x100
addi t0, zero, 0x68
sb t0, 0(x1)
addi t0, zero, 0x65
sb t0, 1(x1)
addi t0, zero, 0x6c
sb t0, 2(x1)
sb t0, 3(x1)
addi t0, zero, 0x6f
sb t0, 4(x1)
addi t0, zero, 0x0a
sb t0, 5(x1)
ebreak
