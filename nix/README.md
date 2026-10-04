# nix/

NixOS + Home Manager config, ported 1:1 from `playbooks/fedora.yaml` and the
dotfiles in `config/` (which stay untouched as your backup).

```
nix/
├── flake.nix                  # entry point
├── hosts/nixos/               # system: the playbook's root tasks
│   ├── default.nix            #   docker, fonts, Plasma, user account
│   ├── flatpak.nix            #   flathub + the flatpak app list
│   └── hardware-configuration.nix  # PLACEHOLDER, replace (see below)
├── home/                      # user: the playbook's user tasks + stow
│   ├── default.nix
│   ├── packages.nix           #   git, tmux, fish, helix, bun, uv, ...
│   ├── rust.nix               #   rustup + rust-analyzer, bacon
│   ├── espanso.nix            #   espanso-wayland + the match files
│   ├── vscode.nix             #   VS Code + extensions + settings
│   └── dotfiles.nix           #   what stow used to do
└── dotfiles/                  # copy of config/ (linked into $HOME)
```

## Install

```sh
# 1. install NixOS from the ISO, then copy this repo to the new system
# 2. replace the placeholder hardware config
nixos-generate-config --show-hardware-config > hosts/nixos/hardware-configuration.nix

# 3. build — flakes only see git-tracked files, so stage this folder first
git add nix
sudo nixos-rebuild switch --flake ./nix#nixos
```

Rename things to taste first: the host (`flake.nix` + `hosts/nixos/default.nix`),
the username (`flake.nix` + `home/default.nix`), the timezone.

## How the playbook maps here

| playbook | nix |
|---|---|
| dnf packages (tmux, fish, bat, helix, jq, ...) | `home/packages.nix` |
| `dnf copr enable atim/starship` | `pkgs.starship` |
| lazydocker / bun / uv curl installers | `pkgs.lazydocker`, `pkgs.bun`, `pkgs.uv` |
| `bun install -g opencode-ai` | `pkgs.opencode` |
| rustup + `rustup component add rust-analyzer` | `home/rust.nix` |
| `cargo install bacon` | `pkgs.bacon` (rustup still owns the toolchains) |
| FiraCode + Nerd Font into /usr/share/fonts | `fonts.packages` |
| VS Code repo + `code` + extensions | `programs.vscode` (extensions by hash) |
| ESPANSO section (AppImage) | `services.espanso` → **espanso-wayland** as a user service |
| flathub + flatpak apps | `hosts/nixos/flatpak.nix` |
| `stow config` | `home/dotfiles.nix` — **no stow** |
| commented-out tasks | left out on purpose |

## Rust

rustup is the source of truth for toolchains (not nixpkgs' rustc), so
`rustc`, `cargo`, `rust-analyzer`, `clippy` and `rustfmt` stay in sync per
toolchain. `home/rust.nix` makes sure `stable` + those components exist after
every switch, and `gcc`/`pkg-config` are there so cargo can link. `bacon`
comes from nixpkgs — if you'd rather have it from cargo, delete it there and
run `cargo install bacon`. `rustup update` is still manual.

## Dotfiles

`home/dotfiles.nix` replaces stow with two buckets:

* **linked** (fish, helix, tmux, starship, rofi, hypr, i3, awesome, ...) —
  symlinked read-only from `nix/dotfiles`, edit them there.
* **seeded** (all of the KDE/Plasma state, cosmic, wayfire, zed) — copied to
  `~/.config` once if missing, then the app owns and rewrites them. Plasma
  constantly rewrites `kwinrc`, `plasma-org.kde.plasma.desktop-appletsrc` etc.;
  store symlinks would make it fail to save.

Anything you decide you don't want later: drop it from `linked`/`seeded` (or
from the playbook-derived package list) and delete the file from
`nix/dotfiles`.

## Notes

* VS Code settings live in `nix/dotfiles/.config/Code/User/settings.json` and
  are read straight into `programs.vscode` — one source of truth.
* Espanso's YAML is generated from `home/espanso.nix` (the module's native
  format). The originals under `nix/dotfiles/.config/espanso` are kept for
  reference but not linked.
* The flatpak list is applied by a one-shot service on boot; `flatpak install
  --or-update` is a no-op once the apps are installed.
* LM Studio, Ollama, pip packages and the Hyprland stack stay out, exactly as
  commented in the playbook.
