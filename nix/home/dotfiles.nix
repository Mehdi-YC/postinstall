# Dotfiles from nix/dotfiles (copied from config/, which stays as your backup).
# This replaces stow. Files fall into two buckets:
#
#   * linked — plain text configs you edit here; symlinked read-only.
#   * seeded — apps that rewrite their own config (Plasma, COSMIC, Wayfire,
#              Zed); copied into ~/.config once, then owned by the app, so
#              the app never writes back into the store.
#
# Code/ and espanso/ are not listed: vscode.nix and espanso.nix own those.
{
  config,
  lib,
  ...
}:
let
  dotfiles = ../dotfiles;
  cfgDir = dotfiles + "/.config";

  linked = [
    "awesome"
    "bin"
    "fish"
    "helix"
    "hypr"
    "i3"
    "i3status"
    "rofi"
    "starship.toml"
  ];

  seeded = [
    # Wayfire / COSMIC / Zed rewrite their own config on exit
    "cosmic"
    "wayfire.ini"
    "wf-shell.ini"
    "zed"
    # KDE / Plasma state
    "discoverrc"
    "dolphinrc"
    "kactivitymanagerdrc"
    "kactivitymanagerd-statsrc"
    "katevirc"
    "kcminputrc"
    "kconf_updaterc"
    "kde.conf.kksrc"
    "kded5rc"
    "kded6rc"
    "kdeglobals"
    "kglobalshortcutsrc"
    "konsolerc"
    "konsolesshconfig"
    "krunnerrc"
    "kscreenlockerrc"
    "ktimezonedrc"
    "kwalletrc"
    "kwinoutputconfig.json"
    "kwinrc"
    "kwriterc"
    "partitionmanagerrc"
    "PlasmaDiscoverUpdates"
    "plasma-localerc"
    "plasma-org.kde.plasma.desktop-appletsrc"
    "plasma_workspace.notifyrc"
  ];
in
{
  home.file = {
    ".bashrc".source = dotfiles + "/.bashrc";
    ".tmux.conf".source = dotfiles + "/.tmux.conf";
    ".Xresources".source = dotfiles + "/.Xresources";
    ".agents".source = dotfiles + "/.agents";
  };

  xdg.configFile = lib.listToAttrs (
    map (name: {
      inherit name;
      value.source = cfgDir + "/${name}";
    }) linked
  );

  home.activation.seedMutableConfigs = lib.hm.dag.entryAfter [ "writeBoundary" ] (
    lib.concatMapStrings (name: ''
      if [ ! -e "$HOME/.config/${name}" ]; then
        run mkdir -p "$HOME/.config"
        run cp -r ${cfgDir}/${name} "$HOME/.config/${name}"
        run chmod -R u+w "$HOME/.config/${name}"
      fi
    '') seeded
  );
}
