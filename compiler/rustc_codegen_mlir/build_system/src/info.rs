use std::collections::HashMap;

use crate::config::ConfigInfo;
use crate::utils::get_toolchain;

struct InfoArg {
    show_env: bool,
    config_info: ConfigInfo,
}

impl InfoArg {
    /// Creates a new `InfoArg` instance by parsing command-line arguments.
    fn new() -> Result<Option<Self>, String> {
        let mut info_arg = Self { show_env: false, config_info: ConfigInfo::default() };
        // Skip binary name and the `info` command.
        let mut args = std::env::args().skip(2);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--env" => {
                    info_arg.show_env = true;
                }
                "--help" => {
                    Self::usage();
                    return Ok(None);
                }
                arg => {
                    if !info_arg.config_info.parse_argument(arg, &mut args)? {
                        return Err(format!("Unknown argument `{}`", arg));
                    }
                }
            }
        }
        Ok(Some(info_arg))
    }

    fn usage() {
        println!(
            r#"
`info` command help:

    --env       : Display environment variables
    --help      : Show this help
"#,
        );
        ConfigInfo::show_usage();
    }
}

fn display_info(info_arg: &mut InfoArg) -> Result<(), String> {
    info_arg.config_info.setup_mlir_path()?;

    println!("\n================================================");
    println!("rustc_codegen_mlir build information");
    println!("================================================\n");

    println!("Rust toolchain: {}", get_toolchain()?);
    println!("Build type: {}", info_arg.config_info.channel.as_str());
    println!("Target: {}", info_arg.config_info.target_triple);
    println!("Host: {}", info_arg.config_info.host_triple);
    println!(
        "MLIR path: {}",
        info_arg.config_info.mlir_path.as_ref().unwrap_or(&"Not found".to_string())
    );
    println!("cargo target dir: {}", info_arg.config_info.cargo_target_dir);

    // When called with `--env`.
    if info_arg.show_env {
        let mut env = HashMap::new();
        info_arg.config_info.setup(&mut env)?;
        print!("\n");
        println!("Environment variables:");
        // Trick to sort by key:
        let mut sorted = env.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|e| e.0);
        for (name, val) in sorted {
            println!("{}={}", name, val);
        }
    }

    Ok(())
}

/// Executes the info process.
pub fn run() -> Result<(), String> {
    let mut info_arg = match InfoArg::new()? {
        Some(info_arg) => info_arg,
        None => return Ok(()),
    };
    display_info(&mut info_arg)
}
