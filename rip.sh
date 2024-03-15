#!/bin/bash

# get dvd file path
dvd_file_path=$1

# if dvd file path is empty, search for dvdrom path
if [ -z $dvd_file_path ]; then
    # drive is /dev/sr0 or /dev/sr1 or /dev/sr2 etc.
    dvd_file_path=$(lsblk -l | grep rom | awk '{print $1}')
    dvd_file_path="/dev/$dvd_file_path"
    echo "DVD file path is $dvd_file_path"
fi

# get disk info
./venv/bin/python main.py --scan -i $dvd_file_path

current_timestamp=$(date +%s)

# rip dvd
./venv/bin/python main.py -i $dvd_file_path -o ./outs/$current_timestamp




