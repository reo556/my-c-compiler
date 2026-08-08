	.section .rodata
.LC0:
	.string "Hello, World!"
	.text
	.intel_syntax noprefix
	.globl main
main:
	lea rax, .LC0[rip]
	push rax
	pop rdi
	mov eax, 0
	call puts
	push rax
	pop rax
	ret
