.section .text
.global _start
.arm

_start:
    mov r0, #0
0:
    add r0, #1
    cmp r0, #100
    bne 0b
1:
    b 1b
