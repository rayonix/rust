use crate::config::ConfigInfo;

pub fn run() -> Result<(), String> {
    // Skip binary name and the `prepare` command.
    let arg = std::env::args().skip(2).next();

    if let Some("--help") = arg.as_deref() {
        usage();
        return Ok(());
    } else if let Some(arg) = arg {
        return Err(format!("Unknown argument `{}`", arg));
    }

    // For now, we don't need any specific preparation steps
    // for rustc_codegen_mlir. This can be extended later if
    // we need to download or set up dependencies.

    Ok(())
}

fn usage() {
    println!(
        r#"
`prepare` command help:

    --help     : Show this help
"#,
    );
    ConfigInfo::show_usage();
}
