./d.f.a_server.sh

#default packages : maybe we add libreoffice , gimp and inkscape
awp='awesome lightdm xfce4-terminal thunar lxappearance dunst compton'

if [ -x "$(command -v pacman)" ];       then 
sudo pacman -Suy  xfce4-terminal thunar lxappearence firefox lightdm awesome
sudo pacman -Suy firefox
elif [ -x "$(command -v apt)" ]; then 
sudo apt install $awp -y
sudo apt install firefox -y
sudo apt install firefox-esr -y
elif [ -x "$(command -v dnf)" ];     then 
sudo dnf install $awp
sudo dnf install firefox
#elif [ -x "$(command -v zypper)" ];  then sudo zypper install $awp
else echo "FAILED TO INSTALL PACKAGE: Package manager not found. You must manually install: $packagesNeeded">&2; fi

sudo systemctl enable lightdm


#downloading some good wallpapers
# AWESOME : 

# create backups
mkdir  ~/.config/awesome
mv   ~/.config/awesome ~/.config/awesome.old 
 

#deleting internal files and directories
rm -rf ~/.config/i3/*
rm -rf ~/.config/awesome/*
mkdir  ~/.config/awesome

#start cloning
cp -r .config/awesome ~/.config/
#wallpapers
git clone https://github.com/linuxdotexe/nordic-wallpapers ~/.config/awesome/nordic-wallpapers


#installing the widgets : 
git clone https://github.com/streetturtle/awesome-wm-widgets  ~/.config/awesome/awesome-wm-widgets

#NERD FONT
#echo "[-] Download fonts [-]"
#echo "https://github.com/ryanoasis/nerd-fonts/releases/download/v2.1.0/DroidSansMono.zip"
#wget https://github.com/ryanoasis/nerd-fonts/releases/download/v2.1.0/DroidSansMono.zip
#unzip DroidSansMono.zip -d ~/.fonts
#fc-cache -fv
echo "done!"

#echo -e "installing nerd fonts \n"
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/install_manual.sh)"
#curl -sS https://webi.sh/nerdfont | sh
