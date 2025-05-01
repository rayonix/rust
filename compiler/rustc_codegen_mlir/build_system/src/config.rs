use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env as std_env, fs};

use boml::Toml;
use boml::types::TomlValue;

use crate::utils::{get_os_name, get_sysroot_dir, rustc_version_info, split_args};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Channel {
    #[default]
    Debug,
    Release,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

fn failed_config_parsing(config_file: &Path, err: &str) -> Result<ConfigFile, String> {
    Err(format!("Failed to parse `{}`: {}", config_file.display(), err))
}

#[derive(Default)]
pub struct ConfigFile {
    mlir_path: Option<String>,
}

impl ConfigFile {
    pub fn new(config_file: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(config_file).map_err(|_| {
            format!(
                "Failed to read `{}`. Take a look at `Readme.md` to see how to set up the project",
                config_file.display(),
            )
        })?;
        let toml = Toml::parse(&content).map_err(|err| {
            format!("Error occurred around `{}`: {:?}", &content[err.start..=err.end], err.kind)
        })?;
        let mut config = Self::default();
        for (key, value) in toml.iter() {
            match (key, value) {
                ("mlir-path", TomlValue::String(value)) => {
                    config.mlir_path = Some(value.as_str().to_string())
                }
                ("mlir-path", _) => {
                    return failed_config_parsing(config_file, "Expected a string for `mlir-path`");
                }
                _ => return failed_config_parsing(config_file, &format!("Unknown key `{}`", key)),
            }
        }
        if let Some(mlir_path) = config.mlir_path.as_mut() {
            let path = Path::new(mlir_path);
            *mlir_path = path
                .canonicalize()
                .map_err(|err| {
                    format!("Failed to get absolute path of `{}`: {:?}", mlir_path, err)
                })?
                .display()
                .to_string();
        } else {
            return failed_config_parsing(config_file, "`mlir-path` value must be set");
        }
        Ok(config)
    }
}

#[derive(Default, Debug, Clone)]
pub struct ConfigInfo {
    pub target: String,
    pub target_triple: String,
    pub host_triple: String,
    pub rustc_command: Vec<String>,
    pub run_in_vm: bool,
    pub cargo_target_dir: String,
    pub dylib_ext: String,
    pub sysroot_release_channel: bool,
    pub channel: Channel,
    pub sysroot_panic_abort: bool,
    pub cg_backend_path: String,
    pub sysroot_path: String,
    pub mlir_path: Option<String>,
    config_file: Option<String>,
    cg_mlir_path: Option<PathBuf>,
    pub no_default_features: bool,
    pub backend: Option<String>,
    pub features: Vec<String>,
}

impl ConfigInfo {
    /// Returns `true` if the argument was taken into account.
    pub fn parse_argument(
        &mut self,
        arg: &str,
        args: &mut impl Iterator<Item = String>,
    ) -> Result<bool, String> {
        match arg {
            "--features" => {
                if let Some(arg) = args.next() {
                    self.features.push(arg);
                } else {
                    return Err("Expected a value after `--features`, found nothing".to_string());
                }
            }
            "--target" => {
                if let Some(arg) = args.next() {
                    self.target = arg;
                } else {
                    return Err("Expected a value after `--target`, found nothing".to_string());
                }
            }
            "--target-triple" => match args.next() {
                Some(arg) if !arg.is_empty() => self.target_triple = arg.to_string(),
                _ => {
                    return Err(
                        "Expected a value after `--target-triple`, found nothing".to_string()
                    );
                }
            },
            "--out-dir" => match args.next() {
                Some(arg) if !arg.is_empty() => {
                    self.cargo_target_dir = arg.to_string();
                }
                _ => return Err("Expected a value after `--out-dir`, found nothing".to_string()),
            },
            "--config-file" => match args.next() {
                Some(arg) if !arg.is_empty() => {
                    self.config_file = Some(arg.to_string());
                }
                _ => {
                    return Err("Expected a value after `--config-file`, found nothing".to_string());
                }
            },
            "--release-sysroot" => self.sysroot_release_channel = true,
            "--release" => self.channel = Channel::Release,
            "--sysroot-panic-abort" => self.sysroot_panic_abort = true,
            "--mlir-path" => match args.next() {
                Some(arg) if !arg.is_empty() => {
                    self.mlir_path = Some(arg.into());
                }
                _ => {
                    return Err("Expected a value after `--mlir-path`, found nothing".to_string());
                }
            },
            "--cg_mlir-path" => match args.next() {
                Some(arg) if !arg.is_empty() => {
                    self.cg_mlir_path = Some(arg.into());
                }
                _ => {
                    return Err(
                        "Expected a value after `--cg_mlir-path`, found nothing".to_string()
                    );
                }
            },
            "--use-backend" => match args.next() {
                Some(backend) if !backend.is_empty() => self.backend = Some(backend),
                _ => return Err("Expected an argument after `--use-backend`, found nothing".into()),
            },
            "--no-default-features" => self.no_default_features = true,
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub fn rustc_command_vec(&self) -> Vec<&dyn AsRef<OsStr>> {
        let mut command: Vec<&dyn AsRef<OsStr>> = Vec::with_capacity(self.rustc_command.len());
        for arg in self.rustc_command.iter() {
            command.push(arg);
        }
        command
    }

    pub fn compute_path<P: AsRef<Path>>(&self, other: P) -> PathBuf {
        match self.cg_mlir_path {
            Some(ref path) => path.join(other),
            None => PathBuf::new().join(other),
        }
    }

    pub fn setup_mlir_path(&mut self) -> Result<(), String> {
        // If the user used the `--mlir-path` option, no need to look at `config.toml` content
        // since we already have everything we need.
        if let Some(mlir_path) = &self.mlir_path {
            println!(
                "`--mlir-path` was provided, ignoring config file. Using `{}` as path for MLIR",
                mlir_path
            );
            return Ok(());
        }
        let config_file = match self.config_file.as_deref() {
            Some(config_file) => config_file.into(),
            None => self.compute_path("config.toml"),
        };
        let ConfigFile { mlir_path } = ConfigFile::new(&config_file)?;

        let Some(mlir_path) = mlir_path else {
            return Err(format!("missing `mlir-path` value from `{}`", config_file.display()));
        };
        println!(
            "MLIR path retrieved from `{}`. Using `{}` as path for MLIR",
            config_file.display(),
            mlir_path
        );
        self.mlir_path = Some(mlir_path);
        Ok(())
    }

    pub fn setup(&mut self, env: &mut HashMap<String, String>) -> Result<(), String> {
        env.insert("CARGO_INCREMENTAL".to_string(), "0".to_string());

        if self.mlir_path.is_none() {
            self.setup_mlir_path()?;
        }
        let mlir_path = self.mlir_path.clone().expect(
            "The config module should have emitted an error if the MLIR path wasn't provided",
        );
        env.insert("MLIR_PATH".to_string(), mlir_path.clone());

        if self.cargo_target_dir.is_empty() {
            match env.get("CARGO_TARGET_DIR").filter(|dir| !dir.is_empty()) {
                Some(cargo_target_dir) => self.cargo_target_dir = cargo_target_dir.clone(),
                None => self.cargo_target_dir = "target/out".to_string(),
            }
        }

        let os_name = get_os_name()?;
        self.dylib_ext = match os_name.as_str() {
            "Linux" => "so",
            "Darwin" => "dylib",
            os => return Err(format!("unsupported OS `{}`", os)),
        }
        .to_string();
        let rustc = match env.get("RUSTC") {
            Some(r) if !r.is_empty() => r.to_string(),
            _ => "rustc".to_string(),
        };
        self.host_triple = match rustc_version_info(Some(&rustc))?.host {
            Some(host) => host,
            None => return Err("no host found".to_string()),
        };

        if self.target_triple.is_empty() {
            if let Some(overwrite) = env.get("OVERWRITE_TARGET_TRIPLE") {
                self.target_triple = overwrite.clone();
            }
        }
        if self.target_triple.is_empty() {
            self.target_triple = self.host_triple.clone();
        }
        if self.target.is_empty() && !self.target_triple.is_empty() {
            self.target = self.target_triple.clone();
        }

        let mut linker = None;

        if self.host_triple != self.target_triple {
            if self.target_triple.is_empty() {
                return Err("Unknown non-native platform".to_string());
            }
            linker = Some(format!("-Clinker={}-gcc", self.target_triple));
            self.run_in_vm = true;
        }

        let current_dir =
            std_env::current_dir().map_err(|error| format!("`current_dir` failed: {:?}", error))?;
        let channel = if self.channel == Channel::Release {
            "release"
        } else if let Some(channel) = env.get("CHANNEL") {
            channel.as_str()
        } else {
            "debug"
        };

        let mut rustflags = Vec::new();
        self.cg_backend_path = current_dir
            .join("target")
            .join(channel)
            .join(&format!("librustc_codegen_mlir.{}", self.dylib_ext))
            .display()
            .to_string();
        self.sysroot_path =
            current_dir.join(&get_sysroot_dir()).join("sysroot").display().to_string();
        if let Some(backend) = &self.backend {
            // This option is only used in the rust compiler testsuite. The sysroot is handled
            // by its build system directly so no need to set it ourselves.
            rustflags.push(format!("-Zcodegen-backend={}", backend));
        } else {
            rustflags.extend_from_slice(&[
                "--sysroot".to_string(),
                self.sysroot_path.clone(),
                format!("-Zcodegen-backend={}", self.cg_backend_path),
            ]);
        }

        // This environment variable is useful in case we want to change options of rustc commands.
        // We have a different environment variable than RUSTFLAGS to make sure those flags are
        // only sent to rustc_codegen_mlir and not the LLVM backend.
        if let Some(cg_rustflags) = env.get("CG_RUSTFLAGS") {
            rustflags.extend_from_slice(&split_args(&cg_rustflags)?);
        }
        if let Some(test_flags) = env.get("TEST_FLAGS") {
            rustflags.extend_from_slice(&split_args(&test_flags)?);
        }

        if let Some(linker) = linker {
            rustflags.push(linker.to_string());
        }

        if self.no_default_features {
            rustflags.push("-Csymbol-mangling-version=v0".to_string());
        }

        // FIXME: check if we need any platform-specific flags for MLIR
        if os_name == "Darwin" {
            rustflags.extend_from_slice(&[
                "-Clink-arg=-undefined".to_string(),
                "-Clink-arg=dynamic_lookup".to_string(),
            ]);
        }
        env.insert("RUSTFLAGS".to_string(), rustflags.join(" "));
        // display metadata load errors
        env.insert("RUSTC_LOG".to_string(), "warn".to_string());

        let sysroot = current_dir
            .join(&get_sysroot_dir())
            .join(&format!("sysroot/lib/rustlib/{}/lib", self.target_triple));
        let ld_library_path = format!(
            "{target}:{sysroot}:{mlir_path}/lib",
            target = self.cargo_target_dir,
            sysroot = sysroot.display(),
            mlir_path = mlir_path,
        );
        env.insert("LIBRARY_PATH".to_string(), ld_library_path.clone());
        env.insert("LD_LIBRARY_PATH".to_string(), ld_library_path.clone());
        env.insert("DYLD_LIBRARY_PATH".to_string(), ld_library_path);

        // Add path to MLIR tools in PATH
        let path = std::env::var("PATH").unwrap_or_default();
        env.insert(
            "PATH".to_string(),
            format!("{}/bin{}{}", mlir_path, if path.is_empty() { "" } else { ":" }, path),
        );

        self.rustc_command = vec![rustc];
        self.rustc_command.extend_from_slice(&rustflags);
        self.rustc_command.extend_from_slice(&[
            "-L".to_string(),
            format!("crate={}", self.cargo_target_dir),
            "--out-dir".to_string(),
            self.cargo_target_dir.clone(),
        ]);

        if !env.contains_key("RUSTC_LOG") {
            env.insert("RUSTC_LOG".to_string(), "warn".to_string());
        }
        Ok(())
    }

    pub fn show_usage() {
        println!(
            "\
    --features [arg]       : Add a new feature [arg]
    --target-triple [arg]  : Set the target triple to [arg]
    --target [arg]         : Set the target to [arg]
    --out-dir              : Location where the files will be generated
    --release              : Build in release mode
    --release-sysroot      : Build sysroot in release mode
    --sysroot-panic-abort  : Build the sysroot without unwinding support
    --config-file          : Location of the config file to be used
    --mlir-path            : Location of the MLIR root folder
    --cg_mlir-path         : Location of the rustc_codegen_mlir root folder (used
                             when ran from another directory)
    --no-default-features  : Add `--no-default-features` flag to cargo commands
    --use-backend          : Useful only for rustc testsuite"
        );
    }
}
