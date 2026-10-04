# Flatpak (playbook: flathub + the GUI apps).
#
# NixOS only ships services.flatpak.enable, so the app list is applied by a
# one-shot service: it runs on every boot and `flatpak install --or-update`
# is a no-op once the apps are there.
{
  pkgs,
  lib,
  ...
}:
let
  flathub = "https://dl.flathub.org/repo/flathub.flatpakrepo";

  apps = [
    "com.github.tchx84.Flatseal" # manage flatpak permissions
    "org.kde.kdenlive" # video editing
    "org.inkscape.Inkscape" # vector manipulation
    "com.valvesoftware.Steam"
    "org.onlyoffice.desktopeditors"
    "com.obsproject.Studio" # screen/stream recording
    "io.github.mfat.sshpilot" # SSH connection manager
  ];

  installApps = pkgs.writeShellScript "flatpak-install-apps" ''
    ${pkgs.flatpak}/bin/flatpak remote-add --if-not-exists --system flathub ${flathub}
    ${pkgs.flatpak}/bin/flatpak install --system --or-update --noninteractive flathub ${lib.escapeShellArgs apps}
  '';
in
{
  services.flatpak.enable = true;
  xdg.portal.enable = true; # required by services.flatpak

  systemd.services.flatpak-install-apps = {
    description = "Install the declarative Flatpak app list from Flathub";
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = installApps;
    };
  };
}
