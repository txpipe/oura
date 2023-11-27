{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane/v0.15.0";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    crane,
    utils,
    ...
  }: let
    supportedSystems = ["x86_64-linux" "x86_64-darwin" "aarch64-linux"];
  in
    utils.lib.eachSystem supportedSystems
    (
      system: {
        packages.oura = crane.lib.${system}.buildPackage {
          src = self;
        };
        packages.default = self.packages.${system}.oura;
      }
    );
}
