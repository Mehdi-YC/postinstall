#get my latest config
cd && git clone https://github.com/mehdi-yc/postinstall
stow -vRd "postinstall/distrobox_builder/" -t $HOME config

#pip install howdoi
if [ -x "$(command -v dnf)" ]; then
sudo dnf copr enable atim/starship -y
sudo dnf copr enable atim/nushell -y
sudo dnf install starship nushell -y
fi
