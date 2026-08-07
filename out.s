	.intel_syntax noprefix
	.globl main
main:
	push 1
	push 2
	pop rdi
	pop rax
	add rax, rdi
	push rax
	push 4
	pop rdi
	pop rax
	imul rax, rdi
	push rax
	push 3
	pop rdi
	pop rax
	cqo
	idiv rdi
	push rax
	pop rax
	ret
