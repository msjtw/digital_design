# digital_design

## Logisim Simulations
Directory containing logisim files for simple circuits.

## HDL exercises
My solutions for exercises from the 4th chapter of the Digital Design and Computer Architecture book.

## Multicycle Processor (WIP)
Implementation in System Verilog of simple RISC-V core based on schematics from Digital Design and Computer Architecture book.

## riscv_em
RISC-V (RV32IMA) core emulator written in rust. It's capable of execution of simple stand-alone ELF binaries.

```bash
riscv32-unknown-elf-gcc -o main main.c
./riscv_em main
```
