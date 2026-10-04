# Rust via rustup (playbook: the rustup tasks).
#
# rustup owns the toolchains, so rust-analyzer/clippy/rustfmt are rustup
# components (they match whatever toolchain you're on). bacon is standalone,
# so it comes from nixpkgs instead of `cargo install`.
{
  config,
  lib,
  pkgs,
  ...
}:
{
  home.packages = with pkgs; [
    rustup # rustc, cargo, rustup component add ...
    bacon # background rust checker
    gcc # linker for cargo builds
    pkg-config
  ];

  # Make sure stable + the components exist after every switch. Idempotent;
  # `|| true` keeps a rebuild from failing when you're offline.
  home.activation.rustupToolchain = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
    export RUSTUP_HOME="${config.home.homeDirectory}/.rustup"
    export CARGO_HOME="${config.home.homeDirectory}/.cargo"
    run ${pkgs.rustup}/bin/rustup toolchain install stable --component rust-analyzer clippy rustfmt --no-self-update || true
    run ${pkgs.rustup}/bin/rustup default stable || true
  '';
}
