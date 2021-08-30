// this is a very minimal implementation of an ARMv7 simular in ARMv7 assembly
// it only supports MOV, ADD, CMP and B. it halts when a branch instruction jumps to itself
// the PC is initialized to the address of 'program'

.section .text
.global _start
.arm

// The program starts here in (32-bit) ARM mode.
_start:
    mov r0, #0xf00
    mov r1, #0
    mov r2, #0
    mov r12, #0
    mov r11, #0

loop_clear:
    str r2, [r0], #4
    add r1, #1
    cmp r1, #16
    bne loop_clear

    mov r0, #0xf3c              // initialize R15 (PC) to the address of program
    adr r1, program
    str r1, [r0]

// registers:
// 0x0F00 - R0
// ..
// 0x0F3C - R15

// R12 - CSPR

loop_execute:
    mov r2, #0xf3c              // load PC
    ldr r1, [r2]

    mov r11, r1                 // original PC
    ldr r0, [r1], #4            // fetch instruction

    str r1, [r2]                // increment PC

    and r1, r0, #0xf0000000     // isolate condition field

    cmp r1, #0xe0000000         // AL (always)
    bne cond_equal
    b   execute_branch

cond_equal:
    cmp r1, #0x00000000         // EQ (equal - Z = 1)
    bne cond_notequal
    ands r2, r12, #0x40000000   // Z bit [30]
    beq loop_execute            // if 'Z bit' is 1 (meaning equal), ands would result in 0, and in return set the Z bit to 1, meaning equal
    b   execute_branch

cond_notequal:
    cmp r1, #0x10000000         // NE (not equal - Z = 0)
    bne unknown_condition
    ands r2, r12, #0x40000000   // Z bit [30]
    bne loop_execute            // if 'Z bit' is 1, ands would result in non-zero, and in return set the Z bit to 0, meaning not equal
    b   execute_branch

unknown_condition:
    mov r11, #2
    b   _exit

execute_branch:
    and r1, r0, #0x0e000000     // get instruction class
    cmp r1, #0x0a000000         // branch
    bne execute_nonbranch

    movw r1, #0x0000ffff
    movt r1, #0x00ff
    and r1, r0, r1

    ands r2, r1, #0x00800000    // isolate MSB [23] to get encoded address sign
    orrne r1, r1, #0x3f000000   // sign-extend to 30 bits if set

    mov r1, r1, lsl #2          // encoded address is number of words, so LSL 2

    movw r2, #0x0000fff8
    movt r2, #0xffff            // -8

    cmp r1, r2                  // is branch to self?
    // -8 is branch to self because addresses are encoded as a value relative to PC+8
    beq _exit                   // halt

    mov r0, r11                 // original PC
    add r0, r0, #8              // branch address is relative to PC+8
    add r0, r0, r1

    mov r1, #0xf3c              // store new PC
    str r0, [r1]

    b   loop_execute

execute_nonbranch:
    and r1, r0, #0x0ff00000     // isolate opcode + addressing mode

add_immediate:
    cmp r1, #0x02800000         // add
    bne cmp_immediate

    and r1, r0, #0x000f0000     // source register
    mov r1, r1, lsr #16

    and r2, r0, #0x0000f000     // destination register
    mov r2, r2, lsr #12

    and r3, r0, #0x00000f00     // shift
    mov r3, r3, lsr #7          // LSR 8, LSL 1

    and r4, r0, #0x000000ff     // immediate
    mov r4, r4, lsl r3

    mov r3, #0xf00              // load source register
    add r3, r1, lsl #2
    ldr r3, [r3]

    add r3, r3, r4

    mov r1, #0xf00              // store value
    add r1, r2, lsl #2
    str r3, [r1]

    b   loop_execute

cmp_immediate:
    cmp r1, #0x03500000         // cmp
    bne mov_immediate

    and r1, r0, #0x000f0000     // source register
    mov r1, r1, lsr #16

    and r2, r0, #0x00000f00     // shift
    mov r2, r2, lsr #7           // LSR 8, LSL 1

    and r3, r0, #0x000000ff     // immediate
    mov r3, r3, lsl r2

    mov r2, #0xf00              // load source register
    add r2, r1, lsl #2
    ldr r2, [r2]

    cmp r2, r3
    mrs r12, cpsr               // copy status register

    b   loop_execute

mov_immediate:
    cmp r1, #0x03a00000         // mov
    bne unknown_instruction

    and r1, r0, #0x0000f000     // destination register
    mov r1, r1, lsr #12

    and r2, r0, #0x00000f00     // shift
    mov r2, r2, lsr #7          // LSR 8, LSL 1

    and r3, r0, #0x000000ff     // immediate
    mov r3, r3, lsr r2

    mov r2, #0xf00              // store value
    add r2, r1, lsl #2
    str r3, [r2]

    b   loop_execute

unknown_instruction:
    mov r11, #1

_exit:
    mov r7, #1
    svc #0

.space 256
program:
    mov r2, #0
program_loop:
    add r2, #1
    cmp r2, #100
    bne program_loop
program_exit:
    b program_exit
