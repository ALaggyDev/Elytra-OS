// gcc -masm=intel -static -nostdlib test.c -o test

void _start()
{
    int a = 5;
    a += 10;

    __asm__(
        "mov rax, 0\n\t"
        "mov rdi, 42\n\t"
        "syscall\n\t");
}