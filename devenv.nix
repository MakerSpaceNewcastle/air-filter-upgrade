{pkgs, ...}: {
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
}
