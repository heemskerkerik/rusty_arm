.section .text
.global _start
.arm

_start:
    mov sp, #0x400

    mov r0, #37
    ldr r1, =fibnum
    bl u32tow

    mov r0, #37
    bl fib
    ldr r1, =fiboutcome
    bl u32tow

    ldr r0, =output
    ldr r1, =strings
    bl strcat

    ldr r0, =output
    bl write_stdout

    mov r7, #1      // syscall - exit
    svc #0

// r0       pointer to string
write_stdout:
    mov r1, r0      // pointer to string
    mov r0, #1      // stdout
    mov r7, #4      // syscall - write
    svc #0
    bx lr

// r0, r1  value to divide (DWORD)
// r2 divisor
// out:
// r0 quotient
// r1 remainder
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

// r0   u32 to encode
// r1   where to store value
u32tow:
    push { lr }
    
    adr r2, sizevalues
    mov r3, #10
2:
    ldr r4, [r2], #4
    
    cmp r0, r4
    bge 3f
    sub r3, #1
    b 2b

3:
    str r3, [r1], #4
    sub r3, #1
    add r1, r3, lsl #1

4:
    push { r1 }

    mov r1, #0
    mov r2, #10
    bl udiv

    mov r2, r1
    pop { r1 }
    add r2, #48         // '0' = 48
    strh r2, [r1], #-2

    cmp r0, #0
    bne 4b
    
    pop { lr }
    bx lr

sizevalues:
    .word 0x3b9aca00    // 1_000_000_000
    .word 0x05f5e100    //   100_000_000
    .word 0x00989680
    .word 0x000f4240
    .word 0x000186a0
    .word 0x00002710
    .word 0x000003e8
    .word 0x00000064    //           100
    .word 0x0000000a    //            10
    .word 0x00000000    //             0

// r0   nth fibonacci number to compute
// out:
// r0   nth fibonacci number
fib:
    cmp r0, #2
    bxle lr

    mov r1, #0          // two fibs ago
    mov r2, #1          // one fib ago
    mov r3, #1          // this fib

    mov r4, #2          // counter

5:
    add r3, r2, r1
    mov r1, r2
    mov r2, r3

    add r4, #1
    cmp r4, r0
    blt 5b

    mov r0, r3
    bx lr

// r0   destination address
// r1   pointer to array of pointers to strings
strcat:
    ldr r2, [r1], #4        // number of pointers
    mov r3, r1              // start of array

    mov r4, #0              // total length
    mov r5, #0              // counter

6:
    ldr r6, [r1], #4
    ldr r6, [r6]
    add r4, r6
    add r5, #1
    cmp r5, r2
    bne 6b

    str r4, [r0], #4

    mov r1, r3
    mov r4, #0              // string counter

7:
    mov r5, #0              // length counter
    ldr r6, [r1], #4        // load pointer to string
    ldr r7, [r6], #4        // load length of string

8:
    ldrh r8, [r6], #2
    strh r8, [r0], #2
    add r5, #1
    cmp r5, r7
    bne 8b

    add r4, #1
    cmp r4, r2
    bne 7b

    bx lr

line_part1:         // "The "
    .word 4
    .short 0x54
    .short 0x68
    .short 0x65
    .short 0x20

line_part2:         // "th Fibonacci number is "
    .word 23
    .short 0x74
    .short 0x68
    .short 0x20
    .short 0x46
    .short 0x69
    .short 0x62
    .short 0x6f
    .short 0x6e
    .short 0x61
    .short 0x63
    .short 0x63
    .short 0x69
    .short 0x20
    .short 0x6e
    .short 0x75
    .short 0x6d
    .short 0x62
    .short 0x65
    .short 0x72
    .short 0x20
    .short 0x69
    .short 0x73
    .short 0x20

.align 4
newline:
    .word 2
    .short 0x0a

.align 4
strings:
    .word 5
    .word line_part1
    .word fibnum
    .word line_part2
    .word fiboutcome
    .word newline

.align 4
fibnum:
    .space 4+(2*2)

.align 4
fiboutcome:
    .space 4+(10*2)

.align 4
output:
    .space 4+(40*2)
