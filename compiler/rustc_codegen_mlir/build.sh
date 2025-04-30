#!/bin/bash

# Simple helper script to run the build system commands
# Usage: ./build.sh [command] [args...]

SCRIPT_PATH=$(dirname "$0")
cd "$SCRIPT_PATH" || exit 1

if [ ! -f "./build_system/target/release/build_system" ]; then
    echo "Building build system first..."
    (cd build_system && cargo build --release) || exit 1
fi

./build_system/target/release/build_system "$@"