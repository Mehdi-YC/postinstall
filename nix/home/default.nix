# User environment — everything the old playbook + stow did for $HOME.
{
  imports = [
    ./packages.nix
    ./rust.nix
    ./espanso.nix
    ./vscode.nix
    ./dotfiles.nix
  ];

  home.username = "mehdi";
  home.homeDirectory = "/home/mehdi";

  # https://nixos.wiki/wiki/FAQ/When_do_I_update_stateVersion
  home.stateVersion = "26.05";

  xdg.enable = true;
  programs.home-manager.enable = true;
}
