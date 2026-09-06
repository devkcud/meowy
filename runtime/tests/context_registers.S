.text
.globl meowy_test_registers_v0
.hidden meowy_test_registers_v0
.type meowy_test_registers_v0,@function
meowy_test_registers_v0:
    pushq %rbp
    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    subq $24, %rsp
    movq %rdi, (%rsp)
    movq %rsi, 8(%rsp)
    movl $0x12121212, %r12d
    movl $0x13131313, %r13d
    movl $0x14141414, %r14d
    movl $0x15151515, %r15d
    movl $0x3b3b3b3b, %ebx
    movl $0x6b6b6b6b, %ebp
    call *8(%rsp)
    testl %eax, %eax
    jnz 1f
    cmpq $0x12121212, %r12
    jne 1f
    cmpq $0x13131313, %r13
    jne 1f
    cmpq $0x14141414, %r14
    jne 1f
    cmpq $0x15151515, %r15
    jne 1f
    cmpq $0x3b3b3b3b, %rbx
    jne 1f
    cmpq $0x6b6b6b6b, %rbp
    jne 1f
    xorl %eax, %eax
    jmp 2f
1:
    movl $1, %eax
2:
    addq $24, %rsp
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    popq %rbp
    ret
.size meowy_test_registers_v0,.-meowy_test_registers_v0
.section .note.GNU-stack,"",%progbits
