{
  description = "Your Rust project description";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          inherit src;
          buildInputs = with pkgs; [
            libinput
            udev
            xorg.libXrandr
            xorg.libX11
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        touchscreen-gestures = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in
      {
        packages.default = touchscreen-gestures;

        devShells.default = pkgs.mkShell {
          inputsFrom = [ touchscreen-gestures ];
          packages = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-audit
          ];
        };

        # This is what makes it usable as an overlay
        overlays.default = final: prev: {
          touchscreen-gestures = touchscreen-gestures;
        };
      });
}
