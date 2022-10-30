./d.f.a_server.sh

awp='awesome lightdm xfce4-terminal thunar lxappearance firefox picom dunst rofi feh'
i3p='i3 xfce4-terminal thunar lxappearance firefox'

#desktop= with if else
echo -e "\n\nselect your window manager : \n \t 1 - awesomewm \n\t 2 -i3wm\n"
read num

if [[ $num -eq 1 ]]; then
desktop=$awp
else 
desktop=$i3p
fi

if [ -x "$(command -v pacman)" ];       then 
sudo pacman -Suy  xfce4-terminal thunar lxappearence firefox
sudo yay -Suy i3-gaps lightdm awesome

elif [ -x "$(command -v apt-get)" ]; then 
sudo apt-get install $desktop

elif [ -x "$(command -v dnf)" ];     then 
sudo dnf install $desktop

#elif [ -x "$(command -v zypper)" ];  then sudo zypper install $packagesNeeded
else echo "FAILED TO INSTALL PACKAGE: Package manager not found. You must manually install: $packagesNeeded">&2; fi

sudo systemctl enable lightdm


#downloading some good wallpapers
# AWESOME : 

echo -e "\n\nselect your config : \n \t 1 - copycat \n\t 2 - JezerM/dotfiles \n"
read num

if [[ $num -eq 1 ]]; then
# create backups
mkdir  ~/.config/awesome
mv   ~/.config/awesome ~/.config/awesome.old 
mkdir  ~/.config/i3
mv   ~/.config/i3 ~/.config/i3.old 

#deleting internal files and directories
rm -rf ~/.config/i3/*
rm -rf ~/.config/awesome/*
mkdir  ~/.config/awesome
mkdir  ~/.config/i3
#start cloning
git clone --recurse-submodules --remote-submodules --depth 1 -j 2 https://github.com/lcpz/awesome-copycats.git
mv -bv awesome-copycats/{*,.[^.]*} ~/.config/awesome; rm -rf awesome-copycats
cd ~/.config/awesome
cp rc.lua.template rc.lua
cd
https://github.com/yashsriv/i3-config /.config/i3
else 
 git clone --recursive https://github.com/JezerM/dotfiles
cd dotfiles
./install.sh
cd
https://github.com/yashsriv/i3-config /.config/i3
fi




#NERD FONT
echo "[-] Download fonts [-]"
echo "https://github.com/ryanoasis/nerd-fonts/releases/download/v2.1.0/DroidSansMono.zip"
wget https://github.com/ryanoasis/nerd-fonts/releases/download/v2.1.0/DroidSansMono.zip
unzip DroidSansMono.zip -d ~/.fonts
fc-cache -fv
echo "done!"

#echo -e "installing nerd fonts \n"
#curl -sS https://webi.sh/nerdfont | sh
