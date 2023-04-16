mkdir $HOME/.config/
cp -r /usr/local/config/* $HOME/.config/
mv ~/.config/.bashrc ~/.bashrc
cd && source ~/.bashrc

#pip install howdoi

sudo dnf copr enable atim/starship -y
sudo dnf copr enable atim/nushell -y
sudo dnf install starship nushell -y

