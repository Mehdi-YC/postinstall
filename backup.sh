# get empty backup.sh
mkdir -p ~/backup
cat <<EOT >> ~/backup/app_backup_$(date +'%d-%m-%Y').sh
#install flatpak and add flathub : 
if ! [ -x "$(command -v flatpak)" ]; then
  sudo dnf install flatpak
fi
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

#install vscodium : 

if ! [ -x "$(command -v codium)" ]; then
  sudo rpmkeys --import https://gitlab.com/paulcarroty/vscodium-deb-rpm-repo/-/raw/master/pub.gpg
  printf "[gitlab.com_paulcarroty_vscodium_repo]\nname=download.vscodium.com\nbaseurl=https://download.vscodium.com/rpms/\nenabled=1\ngpgcheck=1\nrepo_gpgcheck=1\ngpgkey=https://gitlab.com/paulcarroty/vscodium-deb-rpm-repo/-/raw/master/pub.gpg\nmetadata_expire=1h" | sudo tee -a /etc/yum.repos.d/vscodium.repo
  sudo dnf install codium
fi


# install rust : 
if ! [ -x "$(command -v cargo)" ]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi


#install pip if not installed :
if ! [ -x "$(command -v pip)" ]; then
  sudo dnf install python3-pip
fi

EOT

# listing flatpaks pips and crates to install : 
echo "flatpaks : " >>  app_backup_$(date +'%d-%m-%Y').sh
echo " ---- " >>  app_backup_$(date +'%d-%m-%Y').sh
flatpak list --columns=app | tail -n +2 | while read line; do echo flatpak install ${line}; done >> app_backup_$(date +'%d-%m-%Y').sh

echo "python packages : " >>  app_backup_$(date +'%d-%m-%Y').sh
echo " ---- " >>  app_backup_$(date +'%d-%m-%Y').sh
echo pip install $(pip freeze | cut -d "=" -f1  | sort -h | uniq | sed ':a;N;$!ba;s/\n/ /g')

echo "Rust crates : " >>  app_backup_$(date +'%d-%m-%Y').sh
echo " ---- " >>  app_backup_$(date +'%d-%m-%Y').sh
echo cargo install $( cargo install --list c | awk '/^\w/ { print $1 }'| sort -h | uniq | sed ':a;N;$!ba;s/\n/ /g')

echo "vscodium extentions : " >>  app_backup_$(date +'%d-%m-%Y').sh
echo " ---- " >>  app_backup_$(date +'%d-%m-%Y').sh
codium --list-extensions | while read line; do echo codium --install-extension ${line}; done >> app_backup_$(date +'%d-%m-%Y').sh

cp -r ~/.config/ ~/backup/.config$(date +'%d/%m/%Y')
