# digital_design

## riscv_em
RISC-V core emulator written in rust.

Features:
- ima extensions
- machine, supervisor and user modes
- physical memory protection
- virtual memory 
- ns16550a uart
- minimal plic

What is missing:
- c extension (no compressed instructions)
- external devices like block device

To run it you need to build a buildroot image and link it into a single binary with OpenSBI (FW_PAYLOAD).
Or you can use the image from ```image/Image```.

```bash
cd riscv_em
cargo build -r
./target/release/riscv_em ../image/Image   
```

## instr
It's the decoding function extracted from the main emulator.

