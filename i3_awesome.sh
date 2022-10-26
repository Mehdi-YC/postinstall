./d.f.a_server.sh

awp='awesome lightdm xfce4-terminal thunar lxappearance firefox picom dunst rofi feh'
i3p='i3 xfce4-terminal thunar lxappearance firefox'

#desktop= with if else
echo "select your window manager : \n \t 1 - awesomewm \n\t 2 -i3wm picom dunst rofi feh"
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
