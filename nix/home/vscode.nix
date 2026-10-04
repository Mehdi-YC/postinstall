# VS Code (playbook: the desktop tasks).
#
# settings.json stays the single source of truth: dotfiles/.config/Code/User/settings.json
# is read straight into programs.vscode, so edit it there.
{
  pkgs,
  ...
}:
let
  # Not in nixpkgs; fetched from the VS Code marketplace by hash.
  five-server = pkgs.vscode-utils.buildVscodeMarketplaceExtension {
    mktplcRef = {
      name = "five-server";
      publisher = "yandeu";
      version = "0.4.0";
      hash = "sha256-oN4tZATw87M9XGmG2w0QlMPBE3Csv5jw1gZ+G1QGwqk=";
    };
  };
in
{
  programs.vscode = {
    enable = true;
    package = pkgs.vscode; # the MS build, like the playbook's `code`

    profiles.default = {
      # playbook: "Install VS Code extensions"
      extensions = [
        pkgs.vscode-extensions.pkief.material-icon-theme
        five-server
      ];

      userSettings = builtins.fromJSON (
        builtins.readFile ../dotfiles/.config/Code/User/settings.json
      );
    };
  };
}
