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

Elytra OS features a simple build system powered by [xtask](https://github.com/matklad/cargo-xtask). Cargo can automatically download Rust nightly and the required dependencies.

Run `cargo kernel` for a list of subcommands.

```
The ElytraOS build system

Commands:
  build   Build the kernel without running it
  run     Build and run the kernel in QEMU
  fmt     Run cargo fmt
  clippy  Run cargo clippy
  help    Print this message or the help of the given subcommand(s)
```

To run the kernel in QEMU, use `cargo kernel run`.

```
Build and run the kernel in QEMU

Options:
      --release    Build the kernel with optimizations
      --wsl        Use WSL to run QEMU
      --gdb        Listen and wait for a GDB connection in QEMU
      --nographic  Disable graphical output in QEMU
      --kvm        KVM acceleration (need root privileges)
```

Tested version: `rustc 1.101.0-nightly (12433c307 2026-10-24)`.
