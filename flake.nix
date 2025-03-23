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
        devShell = pkgs.mkShell {
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
        };


        overlays.default = final: prev: {
          touchscreen-gestures = touchscreen-gestures;
        };
      }
    );
}
