.global _start
_start:
    mov r0, #1
    adc r0, #1

    mov r0, #0x80000000
    subs r0, #1

    mov r0, #1
    adc r0, #1

    mov r0, #68
    mov r1, #0
    mov r2, #8
    bl udiv
    mov r7, #1
    svc #0

udiv:
0:  cmp     r1,r2
    movhs   r1,r2
    bxhs    lr
    mvn     r3,#0
1:  adds    r0,r0
    adcs    r1,r1
    cmpcc   r1,r2
    subcs   r1,r2
    orrcs   r0,#1
    lsls    r3,#1
    bne     1b
    bx      lr
