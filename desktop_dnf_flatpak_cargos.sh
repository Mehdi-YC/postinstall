# idea : cli tool that generate a script that installs everything:
 #- install flatpak if not installed
 #- add flathub if not added : flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
 #- install cargo if not installed
 #- install vscode if not installed
 #- install python3-pip if not installed
 #- list apps and for loops to install them
 
 
#list all flatpaks id 
flatpak list --columns=app
#images : https://dl.flathub.org/repo/appstream/x86_64/icons/128x128/{app id}.png

#list all installed rpms # not that usefull : using distrobox
# need hypnotix and lutris only


# list all python packages
pip freeze | cut -d "=" -f1  | sort -h | uniq

# cargo install list
cargo install --list


#my flatpaks :
com.discordapp.Discord
com.mattjakeman.ExtensionManager
com.valvesoftware.Steam
de.haeckerfelix.Fragments
io.dbeaver.DBeaverCommunity
io.freetubeapp.FreeTube
io.podman_desktop.PodmanDesktop
net.cozic.joplin_desktop
net.pcsx2.PCSX2
org.gnome.Characters
org.gnome.Music
org.gnome.Totem
org.gnome.Totem.Codecs
org.inkscape.Inkscape
org.onlyoffice.desktopeditors
org.pitivi.Pitivi
org.pitivi.Pitivi.Codecs
org.remmina.Remmina
rest.insomnia.Insomnia

#rpm apps
sudo rpmkeys --import https://gitlab.com/paulcarroty/vscodium-deb-rpm-repo/-/raw/master/pub.gpg
printf "[gitlab.com_paulcarroty_vscodium_repo]\nname=download.vscodium.com\nbaseurl=https://download.vscodium.com/rpms/\nenabled=1\ngpgcheck=1\nrepo_gpgcheck=1\ngpgkey=https://gitlab.com/paulcarroty/vscodium-deb-rpm-repo/-/raw/master/pub.gpg\nmetadata_expire=1h" | sudo tee -a /etc/yum.repos.d/vscodium.repo
sudo dnf install codium



# vscode extantions : code --install-extension {ext name}
jdinhlife.gruvbox
ms-python.python
ms-toolsai.jupyter
ms-toolsai.jupyter-keymap
ms-toolsai.jupyter-renderers
ms-toolsai.vscode-jupyter-cell-tags
ms-toolsai.vscode-jupyter-slideshow
PKief.material-icon-theme
rust-lang.rust
rust-lang.rust-analyzer
s-nlf-fh.glassit
sainnhe.gruvbox-material
svelte.svelte-vscode
tamasfe.even-better-toml
vscodevim.vim
yandeu.five-server

