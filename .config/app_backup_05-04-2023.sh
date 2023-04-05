#install flatpak and add flathub : 
if ! [ -x "/usr/bin/flatpak" ]; then
  sudo dnf install flatpak
fi
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo


#install vscodium : 
if ! [ -x "/usr/bin/codium" ]; then
  sudo rpmkeys --import https://gitlab.com/paulcarroty/vscodium-deb-rpm-repo/-/raw/master/pub.gpg
  printf "[gitlab.com_paulcarroty_vscodium_repo]\nname=download.vscodium.com\nbaseurl=https://download.vscodium.com/rpms/\nenabled=1\ngpgcheck=1\nrepo_gpgcheck=1\ngpgkey=https://gitlab.com/paulcarroty/vscodium-deb-rpm-repo/-/raw/master/pub.gpg\nmetadata_expire=1h" | sudo tee -a /etc/yum.repos.d/vscodium.repo
  sudo dnf install codium
fi


# install rust : 
if ! [ -x "/home/mehdi/.cargo/bin/cargo" ]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  cargo install  hyperfine mdcat zellij miniserve just bat hoard-rs
fi


#install pip if not installed :
if ! [ -x "/usr/bin/pip" ]; then
  sudo dnf install python3-pip
fi

# installing some usefull dnf packages : 
sudo dnf install podman podman-docker jq tmux htop  wget curl neovim git nodejs npm ansible afetch gcc jq unzip zip -y

sudo dnf copr enable alciregi/distrobox
sudo dnf install distrobox

sudo dnf copr enable zeno/scrcpy
sudo dnf install scrcpy


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

#installing starship
curl -sS https://starship.rs/install.sh | sh
cd
source ~/.bashrc
echo 'eval "$(starship init bash)"' >> ~/.bashrc

# starship for nushell
mkdir ~/.cache/starship
sh -c \"$(curl -fsSL https://www.nushell.sh/install.sh)
starship init nu | save ~/.cache/starship/init.nu
echo "source ~/.cache/starship/init.nu" | save --raw --append .config-path

# installing lazydocker (for podman)
curl https://raw.githubusercontent.com/jesseduffield/lazydocker/master/scripts/install_update_linux.sh | bash
sudo cp lazydocker /usr/bin/lazydocker


# listing flatpaks pips and crates to install  : 


#flatpaks : 
flatpak install com.brave.Browser
flatpak install com.discordapp.Discord
flatpak install com.mattjakeman.ExtensionManager
flatpak install com.valvesoftware.Steam
flatpak install de.haeckerfelix.Fragments
flatpak install io.dbeaver.DBeaverCommunity
flatpak install io.freetubeapp.FreeTube
flatpak install io.podman_desktop.PodmanDesktop
flatpak install net.cozic.joplin_desktop
flatpak install net.pcsx2.PCSX2
flatpak install org.fedoraproject.Platform
flatpak install org.freedesktop.Platform
flatpak install org.freedesktop.Platform.Compat.i386
flatpak install org.freedesktop.Platform.ffmpeg-full
flatpak install org.freedesktop.Platform.GL32.default
flatpak install org.freedesktop.Platform.GL.default
flatpak install org.freedesktop.Platform.openh264
flatpak install org.freedesktop.Sdk
flatpak install org.gimp.GIMP
flatpak install org.gnome.Characters
flatpak install org.gnome.Music
flatpak install org.gnome.Platform
flatpak install org.gnome.Totem
flatpak install org.gnome.Totem.Codecs
flatpak install org.gnome.Totem.Videosite.YouTubeDl
flatpak install org.inkscape.Inkscape
flatpak install org.kde.KStyle.Adwaita
flatpak install org.kde.Platform
flatpak install org.kde.PlatformTheme.QGnomePlatform
flatpak install org.kde.PlatformTheme.QtSNI
flatpak install org.kde.WaylandDecoration.QGnomePlatform-decoration
flatpak install org.onlyoffice.desktopeditors
flatpak install org.pitivi.Pitivi
flatpak install org.pitivi.Pitivi.Codecs
flatpak install org.remmina.Remmina
flatpak install rest.insomnia.Insomnia


#python packages : 
pip install ansible ansible-core anyio arandr argcomplete attrs Automat bcrypt beautifulsoup4 Brotli cached-property certifi cffi chardet charset-normalizer cinemagoer click constantly contourpy cryptography cssselect cycler dasbus dbus-python distro dnspython docker docker-compose dockerpty docker-pycreds docopt evdev fastapi filelock file-magic fonttools gpg greenlet hyperlink idna IMDbPY incremental itemadapter itemloaders Jinja2 jmespath jsonschema kiwisolver libcomps libvirt-python lutris lxml MarkupSafe matplotlib mutagen nftables numpy olefile packaging pandas paramiko parsel pexpect Pillow ply polars Protego psutil ptyprocess pyalsa pyasn1 pyasn1-modules pycairo pycparser pycryptodomex pydantic PyDispatcher PyGObject PyNaCl pyOpenSSL pyparsing pypresence pyrsistent PySocks python-augeas python-dateutil python-dotenv pytz PyYAML queuelib requests requests-file resolvelib rpm Scrapy selinux sepolicy service-identity setools setproctitle setroubleshoot six sniffio sos soupsieve SQLAlchemy starlette systemd-python texttable tldextract Twisted typing_extensions Unidecode urllib3 w3lib websocket-client websockets yt-dlp zope.interface


#Rust crates : 
cargo install trunk zellij


#vscodium extentions : 
codium --install-extension jdinhlife.gruvbox
codium --install-extension ms-python.python
codium --install-extension ms-toolsai.jupyter
codium --install-extension ms-toolsai.jupyter-keymap
codium --install-extension ms-toolsai.jupyter-renderers
codium --install-extension ms-toolsai.vscode-jupyter-cell-tags
codium --install-extension ms-toolsai.vscode-jupyter-slideshow
codium --install-extension PKief.material-icon-theme
codium --install-extension rust-lang.rust
codium --install-extension rust-lang.rust-analyzer
codium --install-extension s-nlf-fh.glassit
codium --install-extension sainnhe.gruvbox-material
codium --install-extension svelte.svelte-vscode
codium --install-extension tamasfe.even-better-toml
codium --install-extension vscodevim.vim
codium --install-extension yandeu.five-server