{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.utils.url = "github:numtide/flake-utils";
  inputs.naersk = {
    url = "github:nmattia/naersk";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        naersk-lib = pkgs.callPackage naersk { };
        ropen = naersk-lib.buildPackage { root = ./.; };
      in
      rec
      {
        defaultPackage = ropen;
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

        checks = lib.optionalAttrs (system == "x86_64-linux") {
          int = pkgs.nixosTest ({
            nodes.vm = { ... }: {
              imports = [{ virtualisation.graphics = false; }];
              environment.systemPackages = [ ropen ];
              systemd.services.ropen = {
                wants = [ "network-online.target" ];
                wantedBy = [ "multi-user.target" ];
                serviceConfig = {
                  ExecStart = "${ropen}/bin/server";
                };
              };
            };

            testScript = ''
                start_all()
                vm.wait_for_unit("ropen.service")
                vm.succeed("echo 'echo bar > /tmp/foo' > run.sh")
                vm.succeed("ropen run.sh /bin/sh")
                vm.succeed("cat /tmp/foo") == "bar\n"
            '';
          }
          );
        };
      });
}
