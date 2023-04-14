mkdir $HOME/.config/
cp -r /usr/local/config/* $HOME/.config/


cd # my omt mod
git clone https://github.com/mehdi-yc/.tmux.git
ln -s -f .tmux/.tmux.conf
cp .tmux/.tmux.conf.local .

mv ~/.config/.bashrc ~/.bashrc

# GOOD CLI TOOLS : 
cd && source ~/.bashrc

#pip install howdoi
#cargo install tokei
cargo install --locked hyperfine hoard-rs zellij nu
# curl -sS https://starship.rs/install.sh | sh #eval "$(starship init bash)" currently installed with dnf
sudo dnf copr enable atim/starship -y
sudo dnf install starship -y
# starship for nushell
# mkdir ~/.cache/starship
# nu
# starship init nu | save ~/.cache/starship/init.nu
# echo "source ~/.cache/starship/init.nu" | save --raw --append $nu.config-path
