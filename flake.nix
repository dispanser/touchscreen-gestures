{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

        # Define shared buildInputs and nativeBuildInputs
        buildInputs = with pkgs; [
          libinput
          systemd.dev
          udev
          xorg.libXrandr
          xorg.libX11
        ];

        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          pkg-config
        ];

        touchscreen-gestures = naersk'.buildPackage {
          src = ./.;
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
        };
      in rec {
        # For `nix build` & `nix run`:
        packages.default = touchscreen-gestures;
        # For `nix develop`:
        devShell = pkgs.mkShell.override
          {
            stdenv =
              # use Mold as the linker rather than ld, for faster builds. Mold
              # to require substantially less memory to link Nexus and its
              # avoiding swapping on memory-constrained dev systems.
              pkgs.stdenvAdapters.useMoldLinker
                # use Clang as the C compiler for all C libraries.
                # clangStdenv;
                # stdenv;
                pkgs.llvmPackages.stdenv;
          }
          {
            inputsFrom = [ touchscreen-gestures ];
            packages = with pkgs; [
              clippy
              rustfmt
              rust-analyzer
              cargo-audit
              cargo-nextest
              cargo-watch
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
            RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
            SCCACHE_CACHE_SIZE = "20G";
          };

          overlays.default = final: prev: {
          touchscreen-gestures = touchscreen-gestures;
        };
      }
    );
}
