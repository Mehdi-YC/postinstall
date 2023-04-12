cp -r /usr/local/config/ $HOME/

# my omt mod
cd
git clone https://github.com/mehdi-yc/.tmux.git
ln -s -f .tmux/.tmux.conf
cp .tmux/.tmux.conf.local .

echo "Installing Rust : "
echo 'eval "$(starship init bash)"' >> ~/.bashrc
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable --default-host

# GOOD CLI TOOLS : 
curl -sS https://starship.rs/install.sh | sh #eval "$(starship init bash)"
cd
source ~/.bashrc

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
cargo install nu

# starship for nushell
mkdir ~/.cache/starship
starship init nu | save ~/.cache/starship/init.nu
echo "source ~/.cache/starship/init.nu" | save --raw --append $nu.config-path