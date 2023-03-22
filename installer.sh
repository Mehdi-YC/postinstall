
#packages that i generally use : (maybe adding imagemagick)
packagesNeeded='tmux htop docker-compose python3-pip wget curl neovim git nodejs npm ansible afetch gcc'

if [ -x "$(command -v pacman)" ];       then 
  sudo pacman -Suy $packagesNeeded
  sudo pacman -Suy go  docker
  git clone https://aur.archlinux.org/yay-git.git
  cd yay-git
  makepkg -si

elif [ -x "$(command -v apt)" ]; then 
  sudo apt-get install $packagesNeeded
  sudo apt install golang  docker.io
  sudo apt remove neovim
  sudo apt-get install software-properties-common
  sudo add-apt-repository ppa:neovim-ppa/stable
  sudo apt-get update
  sudo apt install neovim

elif [ -x "$(command -v dnf)" ];     then 
  sudo dnf install $packagesNeeded 
  sudo dnf install go  openssl-devel
  #sudo dnf install docker
  sudo dnf install podman
  
  sudo dnf copr enable varlad/helix
  sudo dnf install helix
  
  sudo dnf copr enable alciregi/distrobox
  sudo dnf install distrobox
  
  sudo dnf copr enable zeno/scrcpy
  sudo dnf install scrcpy


else echo "FAILED TO INSTALL PACKAGE: Package manager not found. You must manually install: $packagesNeeded">&2; fi

# activate docker
sudo systemctl start docker

#nvchad
echo "Installing nvchad"
cd
rm -rf ~/.config/nvim
rm -rf ~/.local/share/nvim
rm -rf ~/.cache/nvim
mkdir .config
git clone https://github.com/NvChad/NvChad ~/.config/nvim --depth 1 && nvim


# my omt mod
cd
git clone https://github.com/mehdi-yc/.tmux.git
ln -s -f .tmux/.tmux.conf
cp .tmux/.tmux.conf.local .

#pyhon packages
python3 -m pip install beautifulsoup4 numpy pandas matplotlib requests fastapi scrapy sqlalchemy


echo "Installing Rust : "
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# GOOD CLI TOOLS : 
curl -sS https://starship.rs/install.sh | sh #eval "$(starship init bash)"
cd
source ~/.bashrc
#lazydocker
curl https://raw.githubusercontent.com/jesseduffield/lazydocker/master/scripts/install_update_linux.sh | bash
sudo cp lazydocker /usr/bin/lazydocker

#reload bash after installing cargo
source ~/.bashrc
bash

#pip install howdoi
#cargo install tokei
cargo install --locked hyperfine
#cargo install bottom
cargo install mdcat
cargo install --locked zellij
cargo install --locked miniserve
cargo install just
cargo install --locked bat
#cargo install --locked broot
#cargo install htmlq
#cargo install skim # grep & fzf
cargo install hoard-rs
#pip install visidata
#python3 -m http.server

# starship for bash
echo 'eval "$(starship init bash)"' >> ~/.bashrc

# starship for nushell
mkdir ~/.cache/starship
starship init nu | save ~/.cache/starship/init.nu
echo "source ~/.cache/starship/init.nu" | save --raw --append $nu.config-path

#echo -e "installing nerd fonts \n"
#/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/install_manual.sh)"

# Wallpapers :
#git clone https://github.com/linuxdotexe/nordic-wallpapers ~/.config/awesome/nordic-wallpapers 



# TODO 
# ADD jupyter notebook with python , rust and nushell ,
# ADD cockpit
# ADD ansible tower 
# ADD portainer
# ADD k9s maybe
#check peregrine and codon
