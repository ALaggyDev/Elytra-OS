use std::{error::Error, path::Path};

use bootloader::BiosBoot;
use clap::Parser;
use xshell::{Shell, cmd};

const KERNEL_PATH: &str = "target/kernel";
const BIOS_PATH: &str = "target/bios.img";

#[derive(Parser)]
#[command(about = "The ElytraOS build system")]
enum Args {
    /// Build the kernel without running it
    Build(BuildArgs),
    /// Build and run the kernel in QEMU
    Run(RunArgs),
    /// Run cargo fmt
    Fmt,
    /// Run cargo clippy
    Clippy,
}

#[derive(Parser)]
struct BuildArgs {
    /// Build the kernel with optimizations
    #[arg(long)]
    release: bool,
}

#[derive(Parser)]
struct RunArgs {
    /// Build the kernel with optimizations
    #[arg(long)]
    release: bool,

    /// Use WSL to run QEMU
    #[arg(long)]
    wsl: bool,

    /// Listen and wait for a GDB connection in QEMU
    #[arg(long)]
    gdb: bool,

    /// Disable graphical output in QEMU
    #[arg(long)]
    nographic: bool,

    /// KVM acceleration (need root privileges)
    #[arg(long)]
    kvm: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let sh = Shell::new()?;

    match args {
        Args::Build(build_args) => {
            build_kernel(&sh, &build_args)?;
        }
        Args::Run(run_args) => {
            build_kernel(
                &sh,
                &BuildArgs {
                    release: run_args.release,
                },
            )?;
            run_kernel(&sh, &run_args)?;
        }
        Args::Fmt => {
            cmd!(sh, "cargo fmt -p kernel").run()?;
        }
        Args::Clippy => {
            cmd!(sh, "cargo clippy -p kernel --target x86_64-unknown-none").run()?;
        }
    }

    Ok(())
}

fn build_kernel(sh: &Shell, args: &BuildArgs) -> Result<(), Box<dyn Error>> {
    let release = args.release.then_some("--release");

    cmd!(
        sh,
        "cargo build {release...} -p kernel --target x86_64-unknown-none"
    )
    .run()?;

    let old_path = Path::new(if args.release {
        "target/x86_64-unknown-none/release/kernel"
    } else {
        "target/x86_64-unknown-none/debug/kernel"
    });

    sh.copy_file(old_path, Path::new(KERNEL_PATH))?;

    BiosBoot::new(Path::new(KERNEL_PATH)).create_disk_image(Path::new(BIOS_PATH))?;

    Ok(())
}

fn run_kernel(sh: &Shell, args: &RunArgs) -> Result<(), Box<dyn Error>> {
    let gdb = if args.gdb { &["-s", "-S"] } else { &[][..] };
    let nographic = args.nographic.then_some("-nographic");
    let kvm = if args.kvm {
        &["-machine", "accel=kvm,type=q35"]
    } else {
        &[][..]
    };

    if !args.wsl {
        cmd!(
            sh,
            "
            qemu-system-x86_64
            -drive format=raw,file={BIOS_PATH}
            -device isa-debug-exit,iobase=0xf4,iosize=0x04
            {nographic...}
            {kvm...}
            {gdb...}
            "
        )
        .run()?;
    } else {
        let kvm_priv = if args.kvm { &["-u", "root"] } else { &[][..] };

        cmd!(
            sh,
            "
            wsl.exe {kvm_priv...} --exec qemu-system-x86_64
            -drive format=raw,file={BIOS_PATH}
            -device isa-debug-exit,iobase=0xf4,iosize=0x04
            {nographic...}
            {kvm...}
            {gdb...}
            "
        )
        .run()?;
    }

    Ok(())
}
