//! xtask — workspace helper for XY-SK120-Modbus-RTU-TTL
//!
//! Usage (from repo root):
//!   cargo xtask build-firmware   — cargo build in xy-sk120-esp-rs/
//!   cargo xtask flash            — cargo build + espflash in xy-sk120-esp-rs/
//!   cargo xtask build-all        — builds the whole workspace then the firmware
//!
//! The firmware crate is excluded from the workspace because its
//! .cargo/config.toml targets bare-metal Xtensa and pins the Espressif
//! toolchain.  This xtask bridges the gap by shelling out with the
//! correct working directory so that firmware-local cargo config is
//! picked up automatically.

use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

fn main() {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("build-firmware") => build_firmware(false),
        Some("flash") => build_firmware(true),
        Some("build-all") => build_all(),
        _ => {
            eprintln!("Usage: cargo xtask <task>");
            eprintln!();
            eprintln!("Tasks:");
            eprintln!("  build-firmware  Build the ESP32-S3 firmware crate");
            eprintln!("  flash           Build and flash the firmware via espflash");
            eprintln!("  build-all       Build the full workspace then the firmware");
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR for xtask is <root>/xtask — go one level up.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must be a subdirectory of the workspace root")
        .to_owned()
}

fn firmware_dir() -> PathBuf {
    workspace_root().join("xy-sk120-esp-rs")
}

/// Run a command and exit with its status code on failure.
fn run(cmd: &mut Command) {
    eprintln!("+ {:?}", cmd);
    let status: ExitStatus = cmd.status().unwrap_or_else(|e| {
        eprintln!("Failed to run command: {e}");
        std::process::exit(1);
    });
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

// ---------------------------------------------------------------------------
// Tasks
// ---------------------------------------------------------------------------

fn build_firmware(flash: bool) {
    let dir = firmware_dir();

    if flash {
        // `cargo run` invokes the runner configured in
        // xy-sk120-esp-rs/.cargo/config.toml (espflash flash --monitor).
        run(Command::new("cargo").args(["run", "--release"]).current_dir(&dir));
    } else {
        run(Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&dir));
    }
}

fn build_all() {
    let root = workspace_root();

    // 1. Build all workspace members (protocol, xtask, future GUI…)
    run(Command::new("cargo").args(["build", "--workspace"]).current_dir(&root));

    // 2. Build the excluded firmware crate.
    build_firmware(false);
}
