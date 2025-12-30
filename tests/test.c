// gcc -masm=intel -static -nostdlib test.c -o test

void _start()
{
    int a = 5;
    a += 10;

    __asm__(
        // yield
        "mov rax, 1\n\t"
        "syscall\n\t"
        // yield
        "mov rax, 1\n\t"
        "syscall\n\t"
        // print exit
        "mov rax, 0\n\t"
        "syscall\n\t");
}