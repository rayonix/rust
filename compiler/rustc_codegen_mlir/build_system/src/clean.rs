use std::fs;
use std::path::Path;

use crate::utils::get_sysroot_dir;

fn clean_sysroot() {
    let path = get_sysroot_dir();
    println!("Cleaning sysroot at {}", path.display());
    let _ = fs::remove_dir_all(path);
}

fn clean_target_out() {
    let path = Path::new("target/out");
    println!("Cleaning target output at {}", path.display());
    let _ = fs::remove_dir_all(path);
}

fn usage() {
    println!(
        r#"
`clean` command help:

    --sysroot    : Clean sysroot only
    --help       : Show this help"#,
    );
}

/// Executes the clean process.
pub fn run() -> Result<(), String> {
    // Skip binary name and the `clean` command.
    let arg = std::env::args().skip(2).next();

    if let Some("--help") = arg.as_deref() {
        usage();
        return Ok(());
    }
    if let Some("--sysroot") = arg.as_deref() {
        clean_sysroot();
    } else if let Some(arg) = arg {
        return Err(format!("Unknown argument `{}`", arg));
    } else {
        // Clean all
        clean_sysroot();
        clean_target_out();
    }
    Ok(())
}
