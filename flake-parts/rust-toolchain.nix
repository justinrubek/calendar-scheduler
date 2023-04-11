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
    ...
  }: let
    fenix-channel = inputs'.fenix.packages.stable;
    # fenix-channel = inputs'.fenix.packages.latest;

    fenix-toolchain = fenix-channel.withComponents [
      "rustc"
      "cargo"
      "clippy"
      "rust-analysis"
      "rust-src"
      "rustfmt"
      "llvm-tools-preview"
    ];
  in rec {
    packages = {
      rust-toolchain = fenix-toolchain;
    };
  };
}
