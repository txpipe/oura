{
  description = "A Nix flake for building oura";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in {
        packages.oura = pkgs.rustPlatform.buildRustPackage {
          name = "oura";
          src = self;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
        packages.default = self.packages.${system}.oura;
        devShells.default = pkgs.mkShell {
          packages = [ rustToolchain ];
        };
      }
    );
}
