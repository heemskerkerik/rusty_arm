.global _start
_start:
	adr r1, memory
	movw r0, #0x5678
	movt r0, #0x1234
	str r0, [r1]
	ldr r2, [r1], #4

	adr r0, memory
	add r1, r0, #4
	movw r2, #0x5678
	movt r2, #0x1234
	//movt r2, #0x1234
	mvn r3, #0
	
	str r2, [r0]
	ldrh r3, [r0]
	mvn r3, #0
	ldrb r3, [r0]

	movw r2, #0x5678
	movt r2, #0x1234

	mvn r3, #0
	str r3, [r1]
	strh r2, [r1]
	str r3, [r1]
	strb r2, [r1]

	mov r7, #1
	svc #0

memory:
	.space 64
