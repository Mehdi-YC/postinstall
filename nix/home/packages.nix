# Packages — 1:1 with the playbook, minus what Nix replaces:
#   * stow              -> Home Manager manages the dotfiles (dotfiles.nix)
#   * atim/starship COPR -> starship straight from nixpkgs
#   * curl | bash installers (lazydocker, bun, uv) -> nixpkgs packages
#   * opencode via bun  -> nixpkgs package
# Everything that was commented out in the playbook is left out on purpose.
{
  pkgs,
  ...
}:
{
  home.packages = with pkgs; [
    # --- minimal (playbook: "Install minimal packages") ---
    git
    tmux
    fish
    bat
    helix

    # --- CLI (playbook: "Install packages") ---
    wget
    curl
    jq
    unzip
    zip
    chromium

    # --- prompt (playbook: "Install starship") ---
    starship

    # --- containers / JS / Python (playbook: curl installers) ---
    lazydocker
    bun
    uv

    # --- AI coding agent (playbook: "Install opencode") ---
    opencode
  ];
}
