{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            gcc-arm-embedded
            pkgsCross.armv7l-hf-multiplatform.stdenv.cc
            pkgsCross.aarch64-multiplatform.buildPackages.gcc

            cmake
            libcxx

            pkg-config

            openssl

            # GUI libs
            libxkbcommon
            libGL
            fontconfig

            # wayland libraries
            wayland
          ];
          LD_LIBRARY_PATH = with pkgs;
            lib.makeLibraryPath [ wayland libxkbcommon libGL fontconfig ];
        };
      });
}
