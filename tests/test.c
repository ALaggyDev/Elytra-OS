// gcc -masm=intel -static -nostdlib test.c -o test

void sys_exit()
{
    asm volatile(
        "mov rax, 0\n\t"
        "syscall\n\t" : : : "rax", "rdi", "rsi", "rdx", "rcx", "r8", "r9", "r10", "r11", "memory");
}

void sys_yield()
{
    asm volatile(
        "mov rax, 1\n\t"
        "syscall\n\t" : : : "rax", "rdi", "rsi", "rdx", "rcx", "r8", "r9", "r10", "r11", "memory");
}

void sys_print(const char *msg, unsigned long len)
{
    asm volatile(
        "mov rax, 2\n\t"
        "syscall\n\t" : : : "rax", "rdi", "rsi", "rdx", "rcx", "r8", "r9", "r10", "r11", "memory");
}

void _start()
{
    sys_yield();

    const char msg[] = "Hello from C user program!\n";
    sys_print(msg, sizeof(msg) - 1);

    sys_exit();
}