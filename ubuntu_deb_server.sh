# update & upgrade
sudo apt update && sudp aot upgrade


# install nvim & nvchad
sudo apt-get install software-properties-common
sudo add-apt-repository ppa:neovim-ppa/stable
sudo apt-get update
sudo apt install neovim vim nano
git clone https://github.com/NvChad/NvChad ~/.config/nvim --depth 1 && nvim
# and install with :tsinstall svelte css javascript...


#install starship maybe
curl -sS https://starship.rs/install.sh | sh

# install tmux & omt modded
sudo apt install tmux -y
cd
git clone https://github.com/mehdi-yc/.tmux.git
ln -s -f .tmux/.tmux.conf
cp .tmux/.tmux.conf.local .


# install python & libraries that i use
sudo apt install python3 python3-pip python3-venv -y
python3 -m pip install beautifulsoup4 numpy pandas matplotlib requests fastapi scrapy sqlalchemy


# install golang node npm 
sudo apt install golang npm nodejs -y


# install docker docker compose and lazydocker
sudo apt install docker.io docker-compose -y
curl https://raw.githubusercontent.com/jesseduffield/lazydocker/master/scripts/install_update_linux.sh | bash
sudo cp lazydocker /usr/bin/lazydocker


# install task warrior
sudo apt-get install taskwarrior -y


# install certbot & nginx
sudo apt install nginx -y
sudo apt install snapd -y
#or with pip : sudo -H pip3 install certbot
sudo snap install --classic certbot
sudo ln -s /snap/bin/certbot /usr/bin/certbot
#sudo certbot --nginx


# install ansibel
sudo apt install ansibel -y

# extra
sudo apt install htop -y
# maybe adding zsh 


#dockers
./dockers.sh


