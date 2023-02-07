
#maybe i can add taskwarrior and rustc
packagesNeeded='tmux htop docker-compose python3-pip wget curl neovim git nodejs npm ansible afetch gcc fzf'

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
  sudo dnf install go  docker openssl-devel
  sudo dnf copr enable varlad/helix
  sudo dnf install helix

elif [ -x "$(command -v zypper)" ];  then 
  sudo zypper install $packagesNeeded

else echo "FAILED TO INSTALL PACKAGE: Package manager not found. You must manually install: $packagesNeeded">&2; fi

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


#install portainer
#./docker.sh


# portainer = all the rest (almost) add confirmation
#sudo docker volume create portainer_data
#sudo docker run -d -p 8000:8000 -p 9443:9443 -p 9000:9000 --name portainer \
#    --restart=always \
#    -v /var/run/docker.sock:/var/run/docker.sock \
#    -v portainer_data:/data \
#    portainer/portainer-ce
# some docker-composes that iv tweaked
#cd 
#git clone https://github.com/Mehdi-YC/mydockerconfigs ./dockers 

echo "Installing Rust : "
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# GOOD CLI TOOLS : 
pip install howdoi
curl -sS https://starship.rs/install.sh | sh #eval "$(starship init bash)"
  #lazydocker
curl https://raw.githubusercontent.com/jesseduffield/lazydocker/master/scripts/install_update_linux.sh | bash
sudo cp lazydocker /usr/bin/lazydocker

cargo install tokei
cargo install --locked hyperfine
cargo install bottom
cargo install mdcat
cargo install --locked zellij
cargo install --locked miniserve
cargo install just
cargo install --locked bat

cargo install --locked broot

cargo install htmlq
cargo install skim # grep & fzf
cargo install hoard-rs
pip3 install visidata
#python3 -m http.server
echo 'eval "$(starship init bash)"' >> ~/.bashrc

mkdir ~/.cache/starship
starship init nu | save ~/.cache/starship/init.nu

echo "source ~/.cache/starship/init.nu"| save --raw --append $nu.config-path

