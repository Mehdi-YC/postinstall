# System configuration — the part of the old playbook that needs root:
# docker, flatpak, fonts, the desktop, and the user account.
{
  pkgs,
  ...
}:
{
  imports = [
    ./hardware-configuration.nix
    ./flatpak.nix
  ];

  # --- Nix ---------------------------------------------------------------
  nix.settings.experimental-features = [
    "nix-command"
    "flakes"
  ];
  nixpkgs.config.allowUnfree = true; # VS Code

  # --- Boot --------------------------------------------------------------
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;
  hardware.enableRedistributableFirmware = true;

  # --- Network / locale --------------------------------------------------
  # TODO: rename the host and set your timezone.
  networking.hostName = "nixos";
  networking.networkmanager.enable = true;
  time.timeZone = "UTC";
  i18n.defaultLocale = "en_US.UTF-8";

  # --- User --------------------------------------------------------------
  users.users.mehdi = {
    isNormalUser = true;
    description = "mehdi";
    extraGroups = [
      "networkmanager"
      "wheel"
      "docker"
    ];
    shell = pkgs.fish;
  };

  # --- Shell -------------------------------------------------------------
  programs.fish.enable = true;

  # --- Docker (playbook: docker, docker-compose) -------------------------
  virtualisation.docker.enable = true;
  environment.systemPackages = [ pkgs.docker-compose ];

  # --- Desktop -----------------------------------------------------------
  # The KDE/Plasma dotfiles in nix/dotfiles assume Plasma 6.
  services.desktopManager.plasma6.enable = true;
  services.displayManager.sddm.enable = true;
  hardware.graphics.enable32Bit = true; # Steam (flatpak) 32-bit runtime

  # --- Fonts (playbook: fira-code-fonts + FiraCode Nerd Font) ------------
  fonts.packages = with pkgs; [
    fira-code
    nerd-fonts.fira-code
  ];

  # --- Misc --------------------------------------------------------------
  # Lets unpackaged binaries (rustup toolchains, downloaded tools) find
  # their shared libraries on NixOS.
  programs.nix-ld.enable = true;

  system.stateVersion = "26.05";
}
