#!/bin/sh
glibcPath="$(nix-build '<nixpkgs>' -A glibc)"

ldLibPath="${glibcPath}/lib"
ldPath="${glibcPath}/lib/ld-linux-aarch64.so.1"

patchelf --set-interpreter $ldPath ./rgb-2025-unwrapped

LD_LIBRARY_PATH=${ldLibPath} ./rgb-2025-unwrapped