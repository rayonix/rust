use std::ffi::OsStr;

use crate::utils::run_command;

pub fn run() -> Result<(), String> {
    // Skip binary name and the `rustc-info` command.
    let mut args = std::env::args().skip(2);
    let arg = args.next();

    if let Some("--help") = arg.as_deref() {
        usage();
        return Ok(());
    }

    println!("\n================================================");
    println!("rustc information");
    println!("================================================\n");

    let command: Vec<&dyn AsRef<OsStr>> = vec![&"rustc", &"-vV"];
    run_command(&command, None)?;

    Ok(())
}

fn usage() {
    println!(
        r#"
`rustc-info` command help:

    --help     : Show this help
"#,
    );
}
