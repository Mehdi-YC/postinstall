./d.f.a_server.sh

awp='awesome lightdm xfce4-terminal thunar lxappearance firefox picom dunst compton'

#desktop= with if else

if [ -x "$(command -v pacman)" ];       then 
sudo pacman -Suy  xfce4-terminal thunar lxappearence firefox lightdm awesome

elif [ -x "$(command -v apt)" ]; then 
sudo apt install $awp

elif [ -x "$(command -v dnf)" ];     then 
sudo dnf install $awp

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
cp -r .config/awesome ~/.config/awesome





#NERD FONT
#echo "[-] Download fonts [-]"
#echo "https://github.com/ryanoasis/nerd-fonts/releases/download/v2.1.0/DroidSansMono.zip"
#wget https://github.com/ryanoasis/nerd-fonts/releases/download/v2.1.0/DroidSansMono.zip
#unzip DroidSansMono.zip -d ~/.fonts
#fc-cache -fv
echo "done!"

#echo -e "installing nerd fonts \n"
#curl -sS https://webi.sh/nerdfont | sh
