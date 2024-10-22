#!/bin/bash

# get dvd file path
dvd_file_path=$1
current_timestamp=$(date +%s)

needed_packages="handbrake-cli python3 vlc sshpass"

os_name=$(cat /etc/os-release | grep ID_LIKE | awk -F'=' '{print $2}' | tr -d '"')
echo "OS name is $os_name"

# check if distro is debian based
if [ "$os_name" = "debian" ]; then
    echo "Debian based distro"
    # ask for confirmation
    read -p "Install the following packages: $needed_packages? [y/n]: " confirm
    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ] || [ "$confirm" = "yes" ]; then
        #sudo apt update
        sudo apt install -y $needed_packages
    fi
# else if arch based
elif [ "$os_name" = "arch" ]; then
    echo "Arch based distro"
    # ask for confirmation
    read -p "Install the following packages: $needed_packages? [y/n]: " confirm
    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ] || [ "$confirm" = "yes" ]; then
        #sudo pacman -Syu
        sudo pacman -S --noconfirm $needed_packages
    fi
else
    echo "Unsupported distro? Please install the following packages: $needed_packages"
fi

# if dvd file path is empty, search for dvdrom path
if [ -z $dvd_file_path ]; then
    # drive is /dev/sr0 or /dev/sr1 or /dev/sr2 etc.
    dvd_file_path=$(lsblk -l | grep rom | awk '{print $1}')
    dvd_file_path="/dev/$dvd_file_path"
    echo "DVD file path is $dvd_file_path"
fi

# check if dvd is mounted
mount_point=$(df -h | grep $dvd_file_path | awk '{print $6}')
if [ -z $mount_point ]; then
    echo "DVD is not mounted"
    echo "Mounting DVD"
    mount_point="/media/dvd"
    # if something is already mounted at /media/dvd, then use /media/dvd1, /media/dvd2 etc.
    # check if /media/dvd is already mounted
    #if [ -d $mount_point ]; then
        #echo "Mount point $mount_point exists"
        #i=0
        #while [ -d $mount_point ]; do
        #    mount_point="/media/dvd$(($i+1))"
        #done
    #fi
    # check if mount point exists
    if [ -d $mount_point ]; then
        echo "Mount point $mount_point exists"
    else
        echo "Creating mount point $mount_point"
        sudo mkdir -p $mount_point
    fi
    sudo mount $dvd_file_path $mount_point
fi
echo "DVD is mounted at $mount_point"

read -p 'Movie Name: ' dname

# get disk info
echo "Getting disk info"
./venv/bin/python main.py --scan -i $dvd_file_path > ./outs/$current_timestamp.nfo
echo "Disk info saved at ./outs/$current_timestamp.nfo\nErrors above can be safely ignored"

# rip dvd
echo "Ripping DVD"
./venv/bin/python main.py -i $dvd_file_path -o ./outs/$current_timestamp
echo "DVD ripped into ./outs/$current_timestamp"

echo "Unmounting DVD"
sudo umount $dvd_file_path
echo "DVD unmounted"


servip=
serverpass=""
dnum=1

echo "Renaming and copying files"
for f in ./outs/1711117007/*
do
  if [ -f "$f" ]
  then
    echo "Processing $f"
    mkdir --parents "./outs/$dname/"
    mv -f "$f" "./outs/$dname/$dname - $dnum.mp4"
    echo "Copying $f to $servip"
    cd "./outs"
    sshpass -p $serverpass rsync -aR "./$dname/$dname - $dnum.mp4" jellyfin@$servip:"/mrd/media/movies/"
    cd ..
    dnum=$((dnum+1))
  else
    echo "Warning: Some problem with \"$f\""
  fi
done





echo "DONE!"
exit 0
