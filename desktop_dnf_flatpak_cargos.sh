#list all flatpaks id 
flatpak list --columns=app
#images : https://dl.flathub.org/repo/appstream/x86_64/icons/128x128/{app id}.png

#list all installed rpms # not that usefull : using distrobox



# list all python packages
pip freeze | cut -d "=" -f1  | sort -h | uniq

# cargo install list
cargo install --list

