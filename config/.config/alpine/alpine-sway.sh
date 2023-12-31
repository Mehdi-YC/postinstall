#!/bin/bash

ALPUSER="mehdi"
ALPNAME="mehdi"

echo "========== Alpine Linux Setup  ================"

echo "====================> Create new user"
adduser -g "$ALPNAME" $ALPUSER

echo "====================> Add user $ALPUSER to proper groups"
adduser $ALPUSER wheel
adduser $ALPUSER input
adduser $ALPUSER video
adduser $ALPUSER audio

echo "====================> Running setup-xorg-base"
setup-xorg-base

echo "====================> Installing helpful packages"
apk add dbus  xdg-desktop-portal-wlr   #xdg-desktop-portal linux-firmware wireless-tools iwd util-linux
#apk add pciutils usbutils coreutils binutils findutils grep iproute2 
#apk add alsa-utils alsa-utils-doc alsa-lib alsaconf alsa-ucm-conf 
apk add dunst rofi
#apk add network-manager-applet kanshi clipman gnome-tweaks gnome-keyring micro vim font-misc-misc 
apk add terminus-font ttf-inconsolata ttf-font-awesome
apk add curl
apk add udisks2 udisks2-doc
apk add mesa-dri-gallium

echo "====================> Installing sway and custom packages"
apk add eudev
setup-devd udev
apk add waybar foot ranger htop neofetch fish tmux starship helix 
apk add sway sway-doc # xwayland 
apk add swaylock swaylockd swaybg swayidle 
apk add ttf-dejavu elogind polkit-elogind #autotiling 

apk add seatd


# now add configs : 

echo "====================> Update main config files"
mkdir /home/$ALPUSER/.config/
cp -r ../.configs/* /home/$ALPUSER/.config/
# cp -r mimeapps.list /etc/xdg/
cat .profile >> /home/$ALPUSER/.profile


echo "====================> Configuring services to launch at boot"
rc-update add seatd
rc-service seatd start
adduser $ALPUSER seat
rc-service dbus start
rc-update add dbus
rc-service iwd start
rc-update add iwd
rc-service alsa start
rc-update add alsa

echo "====================>  Setup complete"
echo "You can now reboot your machine."