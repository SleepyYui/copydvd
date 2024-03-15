#!/bin/bash

# get dvd file path
dvd_file_path=$1
current_timestamp=$(date +%s)

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
    mount_point="/media/dvd/$current_timestamp"
    # check if mount point exists
    if [ -d $mount_point ]; then
        echo "Mount point $mount_point exists"
    else
        echo "Creating mount point $mount_point"
        sudo mkdir -p $mount_point
    fi
    sudo mkdir -p $mount_point
    sudo mount $dvd_file_path $mount_point
fi
echo "DVD is mounted at $mount_point"

# get disk info
./venv/bin/python main.py --scan -i $dvd_file_path

# rip dvd
./venv/bin/python main.py -i $dvd_file_path -o ./outs/$current_timestamp




