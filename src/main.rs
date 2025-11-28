use clap::Parser;
use pathdiff::diff_paths;
use std::env;
use std::path::Path;
use std::process::Command;

// NOTE: Use Ctrl+A x to exit QEMU!

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Use WSL to run QEMU and GDB
    #[arg(long)]
    wsl: bool,

    /// Use GDB to debug the kernel
    #[arg(long)]
    gdb: bool,
}

/// Convert Windows path to relative path (that can be used in WSL)
fn fix_wsl_path(path: &str) -> String {
    diff_paths(Path::new(path), env::current_dir().unwrap())
        .unwrap()
        .to_str()
        .unwrap()
        .replace("\\", "/")
}

fn main() {
    let args = Args::parse();

    // Read env variables that were set in build script
    let kernel_path = env!("KERNEL_PATH");
    let bios_path = env!("BIOS_PATH");

    println!("Kernel is located at: {}", kernel_path);
    println!("Bios image is located at: {}", bios_path);

    // Setup QEMU command
    let mut cmd;
    if !args.wsl {
        cmd = Command::new("qemu-system-x86_64");
    } else {
        cmd = Command::new("wsl.exe");
        cmd.arg("--exec").arg("qemu-system-x86_64");
    }

    // Use serial as output device and disable graphical output
    cmd.arg("-nographic");
    // Enable the guest to exit qemu
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    // Enable GDB if enabled
    if args.gdb {
        cmd.arg("-s").arg("-S");
    }

    // Pass bios paths
    cmd.arg("-drive")
        .arg(format!("format=raw,file={}", fix_wsl_path(bios_path)));

    // Start QEMU
    let mut child = cmd.spawn().expect("failed to start qemu-system-x86_64");

    // Start GDB if enabled
    if args.gdb {
        let mut gdb_cmd;
        if !args.wsl {
            gdb_cmd = Command::new("gdb");
        } else {
            gdb_cmd = Command::new("cmd.exe");
            gdb_cmd.args(["/C", "start", "wsl.exe", "--exec", "gdb"]);
        }
        gdb_cmd.args([
            "-ex",
            &format!("file {} -o 0xffffffff80000000", fix_wsl_path(kernel_path)),
        ]);
        gdb_cmd.args(["-x", "script.gdb"]);

        gdb_cmd.spawn().expect("failed to start gdb");
    }

    let status = child.wait().expect("failed to wait on qemu");
    print!("QEMU exited: {}\n", status);
}
