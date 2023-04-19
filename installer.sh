#packages that i generally use : (maybe adding imagemagick and ffmpeg)
# for desktop add : i3 awesome lightdm xfce4-terminal thunar lxappearance dunst picom and #cargo install i3-style
packagesNeeded='cargo rustc tmux htop docker-compose python3-pip wget curl neovim git nodejs npm ansible afetch gcc jq unzip zip'

if [ -x "$(command -v dnf)" ];     then 
  sudo dnf copr enable alciregi/distrobox -y
  sudo dnf copr enable zeno/scrcpy -y
  sudo dnf copr enable atim/starship -y
  sudo dnf install --setopt=tsflags=nodocs --setopt=install_weak_deps=False $packagesNeeded starship scrcpy distrobox podman podman-docker go  openssl-devel gcc-c++ -y
else echo "FAILED TO INSTALL PACKAGE: Package manager not found. You must manually install: $packagesNeeded">&2; fi

# activate docker
sudo systemctl start docker

cd && echo "Installing nvchad"
rm -rf ~/.config/nvim ~/.local/share/nvim ~/.cache/nvim
mkdir ~/.config
git clone https://github.com/NvChad/NvChad ~/.config/nvim --depth 1

cd && git clone https://github.com/mehdi-yc/.tmux.git
ln -s -f .tmux/.tmux.conf
cp .tmux/.tmux.conf.local .

#pyhon packages
python3 -m pip install beautifulsoup4 numpy pandas matplotlib requests fastapi scrapy

#echo "Installing Rust : "   now installed with the package manager
#  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cd && wget https://raw.githubusercontent.com/Mehdi-YC/postinstall/main/distrobox_builder/.config/.bashrc
source ~/.bashrc

# GOOD CLI TOOLS : 
cargo install tokei hyperfine mdcat zellij just bat hoard-rs nu
#cargo install --locked broot htmlq skim
#pip install visidata howdoi
#python3 -m http.server

# Wallpapers :
#git clone https://github.com/linuxdotexe/nordic-wallpapers ~/.config/awesome/nordic-wallpapers 
wget https://github.com/ryanoasis/nerd-fonts/releases/download/v2.3.3/SourceCodePro.zip
mkdir ~/.local/share/fonts
unzip SourceCodePro.zip -d ~/.local/share/fonts/
fc-cache ~/.local/share/fonts
rm -rf SourceCodePro.zip

#lazydocker
cd && curl https://raw.githubusercontent.com/jesseduffield/lazydocker/master/scripts/install_update_linux.sh | sh

# curl -sS https://starship.rs/install.sh | sh #eval "$(starship init bash)" NEW: installed with dnf
# TODO : ADD jupyter notebook with python , rust and nushell , cockpit  ansible-tower   portainer  k9s
#check peregrine and codon
