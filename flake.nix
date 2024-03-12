{
  description = "A Nix flake for building the oura project on the main branch";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlay ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in {
        packages.oura = pkgs.rustPlatform.buildRustPackage rec {
          name = "oura";
          src = self;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "pallas-0.23.0" = "7deb0f9c183c39d24499f123b17372394385a159ee6380df72fc27335cfa28e8"; 
            };
          };
          cargoSha256 = "0000000000000000000000000000000000000000000000000000"; # Placeholder, replace with actual hash
          buildInputs = with pkgs; [ ];
        };

        defaultPackage = self.packages.oura;
      }
    );
}

