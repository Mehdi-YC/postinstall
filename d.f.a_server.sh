
#maybe i can add taskwarrior and rustc
packagesNeeded='tmux htop docker docker-compose python3-pip wget curl neovim git nodejs npm ansibel neofetch'
if [ -x "$(command -v pacman)" ];       then 
sudo pacman -Suy $packagesNeeded go

elif [ -x "$(command -v apt)" ]; then 
sudo apt-get install $packagesNeeded golang
sudo apt remove neovim
sudo apt-get install software-properties-common
sudo add-apt-repository ppa:neovim-ppa/stable
sudo apt-get update
sudo apt install neovim

elif [ -x "$(command -v dnf)" ];     then 
sudo dnf install $packagesNeeded go

elif [ -x "$(command -v zypper)" ];  then 
sudo zypper install $packagesNeeded

else echo "FAILED TO INSTALL PACKAGE: Package manager not found. You must manually install: $packagesNeeded">&2; fi

#nvchad
git clone https://github.com/NvChad/NvChad ~/.config/nvim --depth 1 && nvim

# my omt mod
cd
git clone https://github.com/mehdi-yc/.tmux.git
ln -s -f .tmux/.tmux.conf
cp .tmux/.tmux.conf.local .

#pyhon packages
python3 -m pip install beautifulsoup4 numpy pandas matplotlib requests fastapi scrapy sqlalchemy

#lazydocker
curl https://raw.githubusercontent.com/jesseduffield/lazydocker/master/scripts/install_update_linux.sh | bash
sudo cp lazydocker /usr/bin/lazydocker

#install portainer
./docker.sh


