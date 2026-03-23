# Elytra OS

My toy kernel written in Rust. This is my second attempt at writing a kernel, the first one is [here](https://github.com/ALaggyDev/toy-kernel).

![GUI demo](images/GUI%20Demo.png)

![Serial output](images/Serial%20Output.png)

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
-   [x] GUI
    -   [x] Wallpaper
    -   [x] Image rendering
    -   [x] Text rendering
    -   [ ] Input system
    -   [ ] Window management
-   [ ] Hardware drivers
-   [ ] Security

# Running

To run Elytra OS using QEMU, you can use the following command. Cargo will automatically download Rust nightly and the required dependencies.

```sh
cargo run -- [--wsl] [--gdb] [--kvm]
```

-   `--wsl`: Run QEMU in WSL. You probably want to use this if you're on Windows.
-   `--gdb`: Automatically start GDB and connect it to QEMU.
-   `--kvm`: Use KVM for hardware acceleration. This is highly recommended as it will significantly improve performance. Requires root privileges.
