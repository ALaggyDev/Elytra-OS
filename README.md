# Elytra OS

My toy kernel written in Rust. This is my second attempt at writing a kernel, the first one is [here](https://github.com/ALaggyDev/toy-kernel).

Progress:

-   [x] Serial output
-   [x] GDT
-   [x] IDT
-   [x] Buddy allocator
-   [x] Slab allocator
-   [x] Paging
-   [x] User mode
-   [x] Tasks & context switch
    -   [x] Cooperative multi-tasking
    -   [ ] Preemptive multi-tasking
    -   [ ] Inactive tasks and wakeup
-   [x] ELF loading
-   [x] Syscalls
-   [ ] Interrupt handling
-   [ ] Hardware drivers
-   [ ] Security

# Running

To run Elytra OS using QEMU, you can use the following command: (WSL and GDB are optional flags)

```sh
cargo run -- --wsl --gdb
```

Cargo will automatically download Rust nightly and the required dependencies.
