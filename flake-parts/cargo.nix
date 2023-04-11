{
  inputs,
  self,
  ...
} @ part-inputs: {
  imports = [];

  perSystem = {
    pkgs,
    lib,
    system,
    inputs',
    self',
    ...
  }: let
    craneLib = inputs.crane.lib.${system}.overrideToolchain self'.packages.rust-toolchain;

    common-build-args = rec {
      src = inputs.nix-filter.lib {
        root = ../.;
        include = [
          "crates"
          "Cargo.toml"
          "Cargo.lock"
        ];
      };

      pname = "scheduler";

      buildInputs = allBuildInputs [];
      nativeBuildInputs = allNativeBuildInputs [];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
    };
    deps-only = craneLib.buildDepsOnly ({} // common-build-args);

    checks = {
      clippy = craneLib.cargoClippy ({
          cargoArtifacts = deps-only;
          cargoClippyExtraArgs = "--all-features -- --deny warnings";
        }
        // common-build-args);

      rust = craneLib.cargoFmt ({
          inherit (common-build-args) src;
        }
        // common-build-args);

      tests = craneLib.cargoNextest ({
          cargoArtifacts = deps-only;
          partitions = 1;
          partitionType = "count";
        }
        // common-build-args);

      pre-commit-hooks = inputs.pre-commit-hooks.lib.${system}.run {
        inherit (common-build-args) src;
        hooks = {
          alejandra.enable = true;
          rustfmt.enable = true;
        };
      };
    };

    packages = rec {
      default = cli;
      cli = craneLib.buildPackage ({
          pname = "caldav-cli";
          cargoArtifacts = deps-only;
          cargoExtraArgs = "--bin cli";
        }
        // common-build-args);
    };

    devTools = with pkgs; [
      # rust tooling
      self'.packages.rust-toolchain
      bacon
      rustfmt
      cargo-nextest
      # version control
      cocogitto
      inputs'.bomper.packages.cli
      # misc
      pkgs.radicale
    ];

    extraBuildInputs = [
      pkgs.pkg-config
    ];
    extraNativeBuildInputs = [
    ];

    allBuildInputs = base: base ++ extraBuildInputs;
    allNativeBuildInputs = base: base ++ extraNativeBuildInputs;
  in rec {
    inherit checks packages;

    devShells.default = pkgs.mkShell rec {
      buildInputs = allBuildInputs [self'.packages.rust-toolchain] ++ devTools;
      nativeBuildInputs = allNativeBuildInputs [];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
      inherit (self.checks.${system}.pre-commit-hooks) shellHook;
    };

    apps = {
      cli = {
        type = "app";
        program = "${self.packages.${system}.cli}/bin/cli";
      };
      default = apps.cli;
    };
  };
}
