cp -r /usr/local/config/* $HOME/.config/

cd # my omt mod
git clone https://github.com/mehdi-yc/.tmux.git
ln -s -f .tmux/.tmux.conf
cp .tmux/.tmux.conf.local .

cargo install hoard-rs

# starship for nushell
mkdir ~/.cache/starship
nu
starship init nu | save ~/.cache/starship/init.nu
echo "source ~/.cache/starship/init.nu" | save --raw --append $nu.config-path