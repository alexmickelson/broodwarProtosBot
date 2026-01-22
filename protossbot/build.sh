#!/usr/bin/env bash

set -e
# Set up environment for cross-compilation
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar

# Set include path for bindgen - add C++ standard library and GCC include paths
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-w64-mingw32/13-posix/include/c++ -I/usr/lib/gcc/x86_64-w64-mingw32/13-posix/include/c++/x86_64-w64-mingw32 -I/usr/lib/gcc/x86_64-w64-mingw32/13-posix/include -I/usr/x86_64-w64-mingw32/include"

# Build for Windows target
echo "Building..."
cargo build --target x86_64-pc-windows-gnu

# Check if build was successful
if [ -f "target/x86_64-pc-windows-gnu/debug/rustbot.exe" ]; then
    echo "Build successful: target/x86_64-pc-windows-gnu/debug/rustbot.exe"
else
    echo "Build failed!"
    exit 1
fi
