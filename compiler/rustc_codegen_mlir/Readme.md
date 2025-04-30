# rustc_codegen_mlir

An alternative backend for the Rust compiler that emits MLIR IR instead of LLVM IR.

## Setting up

### Prerequisites

1. Make sure you have MLIR installed on your system.
2. Copy the `config.toml.example` file to `config.toml` and update the MLIR path:

``` bash
cp config.toml.example config.toml
```

Then edit `config.toml` to set the correct path to your MLIR installation.

### Building

You can build the project using the build system:

```bash
cd build_system
cargo build --release
cd ..
./build_system/target/release/build_system build
```

This will compile the `rustc_codegen_mlir` crate and prepare it for use with Rust.

### Building with sysroot

To build with a custom sysroot (needed for advanced usage):

```bash
./build_system/target/release/build_system build --sysroot
```

## Running

Once built, you can run the Rust compiler with the MLIR backend:

```bash
./build_system/target/release/build_system rustc --path/to/your/file.rs
```

## Testing

To run the test suite:

```bash
./build_system/target/release/build_system test
```

## Commands

The build system supports several commands:

- `build`: Build the codegen backend
- `clean`: Clean build artifacts
- `prepare`: Prepare the environment
- `test`: Run tests
- `rustc`: Run rustc with the MLIR backend
- `cargo`: Run cargo with the MLIR backend
- `info`: Display build information
- `fmt`: Format the code

Run any command with `--help` to see available options.

## Configuration

The build system can be configured with various options. See the help output of each command for details:

```bash
./build_system/target/release/build_system --help
```
