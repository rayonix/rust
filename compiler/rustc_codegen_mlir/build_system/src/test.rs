use std::collections::HashMap;
use std::ffi::OsStr;

use crate::config::{Channel, ConfigInfo};
use crate::utils::run_command_with_output_and_env;

pub enum TestKind {
    UI,
    Codegen,
    CodegenUnits,
    Incremental,
    MirOpt,
    Assembly,
    Debuginfo,
    Cli,
}

struct TestArg {
    flags: Vec<String>,
    config_info: ConfigInfo,
}

fn usage() {
    println!(
        r#"
`test` command help:

    --nocapture            : Display output when running tests.
    --bless                : Bless test output.
    --skip-codegen         : Skip the codegen backend testing.
    --skip-ui              : Skip the ui testing.
    --skip-standard-lib    : Skip standard library testing.
    --filter [filter]      : Set up a regex to filter tests.
    --exact                : Exact filter match (no regex).
    --quiet                : Don't display information about which test is running.
    --ignored              : Run only ignored tests.
    --include-ignored      : Include ignored tests.
    --test-threads [nr]    : Number of threads to use for testing.
    -Z [flag]              : Add a -Z flag to the compiler.
    --external             : Run external tests too (requires git checkout).

    Below options are exclusive (only one is accepted):

    --suite [suite]        : Run the test suite (check, ui, assembly, incremental, ...).
    --run [file]           : Run a specific test file.
"#
    );
    ConfigInfo::show_usage();
    println!("    --help      : Show this help");
}

impl TestArg {
    /// Creates a new `TestArg` instance by parsing command-line arguments.
    fn new() -> Result<Option<Self>, String> {
        let mut test_arg = Self { flags: Vec::new(), config_info: ConfigInfo::default() };
        // Skip binary name and the `test` command.
        let mut args = std::env::args().skip(2);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" => {
                    usage();
                    return Ok(None);
                }
                "--nocapture"
                | "--bless"
                | "--skip-codegen"
                | "--skip-ui"
                | "--skip-standard-lib"
                | "--exact"
                | "--quiet"
                | "--ignored"
                | "--include-ignored"
                | "--external" => {
                    test_arg.flags.push(arg);
                }
                "--filter" | "--test-threads" | "--suite" | "--run" => {
                    if let Some(next_arg) = args.next() {
                        test_arg.flags.push(arg);
                        test_arg.flags.push(next_arg);
                    } else {
                        return Err(format!("Expected a value after `{}`, found nothing", arg));
                    }
                }
                "-Z" => {
                    if let Some(next_arg) = args.next() {
                        test_arg.flags.push(arg);
                        test_arg.flags.push(next_arg);
                    } else {
                        return Err(format!("Expected a value after `{}`, found nothing", arg));
                    }
                }
                arg => {
                    if !test_arg.config_info.parse_argument(arg, &mut args)? {
                        return Err(format!("Unknown argument `{}`", arg));
                    }
                }
            }
        }
        Ok(Some(test_arg))
    }
}

fn run_tests(args: &mut TestArg) -> Result<(), String> {
    let mut env = HashMap::new();
    args.config_info.setup(&mut env)?;

    let mut command: Vec<&dyn AsRef<OsStr>> = vec![&"cargo", &"test"];
    if args.config_info.channel == Channel::Release {
        command.push(&"--release");
    }
    if args.config_info.no_default_features {
        command.push(&"--no-default-features");
    }
    let flags = args.flags.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    for flag in &flags {
        command.push(flag);
    }
    run_command_with_output_and_env(&command, None, Some(&env))?;
    Ok(())
}

/// Executes the test process.
pub fn run() -> Result<(), String> {
    let mut args = match TestArg::new()? {
        Some(args) => args,
        None => return Ok(()),
    };
    args.config_info.setup_mlir_path()?;
    run_tests(&mut args)?;
    Ok(())
}
