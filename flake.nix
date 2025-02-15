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
          udev
          xorg.libXrandr
          xorg.libX11
        ];

        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          rust-analyzer
          pkg-config
        ];

      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
        };
      }
    );
}
