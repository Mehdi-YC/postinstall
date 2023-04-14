mkdir $HOME/.config/
cp -r /usr/local/config/* $HOME/.config/

# my omt mod
cd
git clone https://github.com/mehdi-yc/.tmux.git
ln -s -f .tmux/.tmux.conf
cp .tmux/.tmux.conf.local .

echo "Installing Rust : "
touch ~/.bashrc
echo 'eval "$(starship init bash)"' >> ~/.bashrc

# GOOD CLI TOOLS : 
curl -sS https://starship.rs/install.sh | sh #eval "$(starship init bash)"
cd
source ~/.bashrc

#pip install howdoi
#cargo install tokei
cargo install --locked hyperfine
cargo install --locked zellij
cargo install hoard-rs
cargo install nu

source ~/.bashrc

# starship for nushell
mkdir ~/.cache/starship
nu
starship init nu | save ~/.cache/starship/init.nu
echo "source ~/.cache/starship/init.nu" | save --raw --append $nu.config-path