#!/bin/sh
glibcPath="$(/nix/store/pw2h8sf4jygiq8x9hkna6y5n2a394nvp-nix-2.26.3/bin/nix-build '<nixpkgs>' -A glibc)"

ldLibPath="${glibcPath}/lib"
ldPath="${glibcPath}/lib/ld-linux-aarch64.so.1"

/nix/store/23216cvbd4cv8w206kxmwgds0h1rkxwd-patchelf-0.15.0/bin/patchelf --set-interpreter $ldPath /home/copepod/rgb-2025-unwrapped

LD_LIBRARY_PATH=${ldLibPath} /home/copepod/rgb-2025-unwrapped