#!/bin/bash

function run {
  if ! pgrep $1 ;
  then
    $@&
  fi
}

#start sxhkd to replace Qtile native key-bindings
run sxhkd -c ~/.config/qtile/sxhkd/sxhkdrc &
#start mpd
[ ! -s ~/.config/mpd/pid ] && mpd &

setxkbmap -option ctrl:nocaps &
clipmenud &
ssh-add &
dunst &
wal -R & 
#starting utility applications at boot time
picom --config $HOME/.config/picom/picom.conf --vsync &
/usr/libexec/polkit-gnome-autentication-agent-1 &
#/usr/lib/xfce4/notifyd/xfce4-notifyd &
#deckmaster -deck ~/.config/deck/main.deck &
playerctld daemon &
xhost +si:localuser:$USER &
~/.fehbg &
vorta &
