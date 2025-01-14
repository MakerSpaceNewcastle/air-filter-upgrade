{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            # Code formatting tools
            treefmt
            alejandra
            mdl
            rustfmt

            # Rust toolchain
            rustup
            probe-rs

            # Extra tools for control program
            cargo-cross

            # Sensor firmware toolchain
            esphome
          ];
        };
      }
    );
}
