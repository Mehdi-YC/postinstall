---
title: Advanced commands
updated: 2022-05-15 14:45:42Z
created: 2022-03-20 11:04:38Z
latitude: 36.72950000
longitude: 3.09050000
altitude: 0.0000
---

## switching from pipewire to pulseaudio : 
```bash
sudo dnf swap --allowerasing pulseaudio pipewire-pulseaudio
```

# THE LINUX COMMANDS

## ==*BASIC*==

`echo` prints something on the terminal

`ls` : list a directory files & folders  `mkdir / touch  & rmdir / rm`  to create & delete

`cd` : change directory

`cat ,head ,tail ,less ,more`: show the content of the file in the terminal

`Alias / unalias` : give another name to a command

`pwd` : prints the current directory path

`whoami / who` : prints the current user

`which / whereis` : check where the command binary is saved

`man` : manual of a command

`find` : find a folder or a file in a directory

`pr -l[x] [file]` : print x number of lines in each page from the file file

* * *

* * *

## ==*USERS & PERMISSIONS*==

`chmod` :change permossion of a file or a directory

`psswd` : change the password for a user

`useradd` : create a new user

* * *

* * *

## ==*WEB*==

`ping ` : ping a server

`wget` : download the content by providing th user

`curl` : show the url result in the terminal

* * *

* * *

## ==*MONITORING*==

`top`

`ps`

`kill`

`pkill`

`watch`

* * *

* * *

## ==*DATA MANIPULATION*==

`grep`

`cut`

`diff`

`patch`

`uniq`

`sort`

`head`

`tail`

`wc`

* * *

* * *

## ==*POWER*==

`uptime`

`reboot`

`shutdown`

`poweroff`

`last`

* * *

* * *

## ==*COMMUNICATION*==

`wall` : broadcast message to all users

`write` : write a mesage to one user

`mesg (y/n)` disable enable messages

* * *

* * *

## ==*Networking*==

### old

`ifconfig`
`route`

### new

`ip`
`ss`

* * *

* * *

## ==STORAGE==

***common places to find devices : /mnt /media /dev***

`lsblk` (list all devices)

`ls /dev/...` devices are files

`sudo fdisk -l` (list infos about storage devices)

`mount` (list all devices)

`mount | grep sdb` (list specefic type of devices)

`sudo umount path/to/drive` (unmount device)

`sudo fdisk` (list of commands ex : creacte partitions)

`sudo mkfs.format  -n name path/to/device` (format a device "-n to give it a name")

`df -h` (show amount of used space)

`blkid`

`du` : space used by  all files / directories

`free` : show informations about ram usage

## unmounting and mounting :

`mkdir /mnt/disk1`

`sudo mount/dev/sdb1 /mnt/disk1`

good packages : `ncdu`

* * *

* * *

## ==*OTHER USEFULL COMMANDS*==

`file`

`type`

`ncp`

`uname`

`diff`

`patch`

`nohup` send a process to background with the ability to resend it to front with fg

* * *

* * *

## ==**`systemd`**==

systemctl
location of configs : `/usr/lib/systemd/system/*.service`
`systemctl status [service]`
`systemctl enable [service]`
`systemctl disable [service]`
`systemctl start [service]`
`systemctl stop [service]`
`systemctl restart [service]`
`systemctl reload [service]`
system ctl enable --now \[srvice\] will enable & start a service (or daemon)
service example : http server : `httpd`

show only services with systemctl
`systemctl list-unit-files --type=service`

whiptail
