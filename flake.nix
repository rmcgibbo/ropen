{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShell = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            cargo
            cargo-udeps
            rustc
            clippy
            rustfmt
            rust-analyzer
          ];
        };
      });
}
