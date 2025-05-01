use crate::utils::{cargo_install, run_command_with_output};

pub fn run() -> Result<(), String> {
    let arg = std::env::args().skip(2).next();

    if let Some("--help") = arg.as_deref() {
        usage();
        return Ok(());
    }
    if let Some(arg) = arg {
        return Err(format!("Unknown argument `{}`", arg));
    }

    cargo_install("rustfmt")?;
    run_command_with_output(&[&"cargo", &"fmt"], None)?;
    Ok(())
}

fn usage() {
    println!(
        r#"
`fmt` command help:

    --help     : Show this help
"#,
    );
}
