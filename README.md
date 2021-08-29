# rusty_arm
Rust implementation of an ARMv7 CPU emulator.

## How to run

* Compile one of the provided 'programs' in the `asm` directory using `compile.sh`, which requires the GNU Arm Embedded Toolchain to be installed.
  * For example: `./compile.sh Fib` will assemble and link `Fib.s` to `Fib.s.elf`.
* Run the emulator using `cargo run`, passing a single argument, the path to the ELF file.
  * E.g. `cargo run ../asm/Fib.s.elf`

## Support
Not all of the ARM ISA has been implemented. Here’s what’s currently implemented:

* All conditions.
* For instructions that support it, setting flags.
* Classic ARM (32-bit instructions) only.
* Launching from ELF binaries
  * This simply looks at whether the binary starts with the ‘ELF magic number’ (`[ 0x7F, 0x45, 0x4C, 0x46, 0x01, 0x01, 0x01, 0x00 ]`), and if so, lets the entry point be whatever address is encountered at offset `0x18` in the file. Otherwise it just starts at zero.

### Instructions
* Moving: `MOV`, `MVN`, `MOVW`, `MOVT`
* Arithmetic: `ADD`, `ADC`, `SUB`
* Branching: `B`, `BL`, `BX`
* Bitwise: `AND`, `ORR`
* Status registers: `CMP`, `MRS`
* Loading & storing: `STR`, `LDR`, `STRH`, `STRB`, `LDRH`, `LDRB`
* Other: `SVC`

### Addressing modes
* For data processing instructions, both shifted immediate and (immediate or register)-shifted register are implemented. `RRX` shifting is not implemented.
* For load/store:
  * Register indirect (`LDR R1, [R0]`)
  * Register with immediate offset (`LDR R1, [R0, #4]`)
  * Register with register offset (`LDR R1, [R0, R2]`)
  * Register with scaled register offset (`LDR R1, [R0, R2, LSL #2]`)
  * Pre-indexed and post-indexed versions of these

### ABI
The ABI implemented is based on the Linux one (system call number in `r7`), but only supports two system calls:
* Exit (`r7 = 1`)
* Write (`r7 = 4`)
  * `r0` is ‘file descriptor’, but only the value `1` (standard output) is supported.
  * `r1` is a pointer to a length-prefixed UTF-16 string to write.
