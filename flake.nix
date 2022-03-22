{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixpkgs-unstable;
    naersk = {
      url = github:nix-community/naersk;
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = github:nix-community/fenix;
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, naersk, fenix }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      toolchain = with fenix.packages.x86_64-linux;
        combine [
          minimal.rustc
          minimal.cargo
          targets.x86_64-unknown-linux-musl.latest.rust-std
        ];

      # Make naersk aware of the tool chain which is to be used.
      naersk-lib = naersk.lib.x86_64-linux.override {
        cargo = toolchain;
        rustc = toolchain;
      };
      # Utility for merging the common cargo configuration with the target
      # specific configuration.
      naerskBuildPackage = target: args: naersk-lib.buildPackage
        (args // { CARGO_BUILD_TARGET = target; } // cargoConfig);
      # All of the CARGO_* configurations which should be used for all
      # targets. Only use this for options which should be universally
      # applied or which can be applied to a specific target triple.
      # This is also merged into the devShell.
      cargoConfig = {
        # Enables static compilation.
        #
        # If the resulting executable is still considered dynamically
        # linked by ldd but doesn't have anything actually linked to it,
        # don't worry. It's still statically linked. It just has static
        # position independent execution enabled.
        # ref: https://doc.rust-lang.org/cargo/reference/config.html#targettriplerustflags
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = "-C target-feature=+crt-static";
      };
      ropen = naerskBuildPackage "x86_64-unknown-linux-musl" {
        src = ./.;
        doCheck = true;
      };
    in
    rec {
      overlay = final: prev: {
        ropen = ropen;
      };
      defaultPackage.x86_64-linux = ropen;
      devShell.x86_64-linux = pkgs.mkShell (rec {
        inputsFrom = [ ropen ];
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
      } // cargoConfig
      );
    };
}
