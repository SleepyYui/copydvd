#!/bin/bash

# get dvd file path
dvd_file_path=$1
current_timestamp=$(date +%s)

needed_packages="handbrake-cli python3 vlc"

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

# get disk info
echo "Getting disk info"
./venv/bin/python main.py --scan -i $dvd_file_path > ./outs/$current_timestamp.nfo
echo "Disk info saved at ./outs/$current_timestamp.nfo"

# rip dvd
echo "Ripping DVD"
./venv/bin/python main.py -i $dvd_file_path -o ./outs/$current_timestamp
echo "DVD ripped into ./outs/$current_timestamp"

echo "Unmounting DVD"
sudo umount $dvd_file_path
echo "DVD unmounted"

echo "DONE!"
exit 0
