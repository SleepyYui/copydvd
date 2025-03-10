#!/bin/bash

# get dvd file path
dvd_file_path=$1
current_timestamp=$(date +%s)

needed_packages="handbrake-cli python3 vlc sshpass rsync"

# Detect OS more efficiently
os_name=$(grep -Po '(?<=^ID_LIKE=).+' /etc/os-release | tr -d '"')
echo "OS name is $os_name"

# Install packages based on distro
if [[ "$os_name" == *"debian"* ]]; then
    echo "Debian based distro detected"
    read -p "Install required packages ($needed_packages)? [y/n]: " confirm
    if [[ "$confirm" =~ ^[Yy](es)?$ ]]; then
        sudo apt-get update -qq && sudo apt-get install -y $needed_packages
    fi
elif [[ "$os_name" == *"arch"* ]]; then
    echo "Arch based distro detected"
    read -p "Install required packages ($needed_packages)? [y/n]: " confirm
    if [[ "$confirm" =~ ^[Yy](es)?$ ]]; then
        sudo pacman -S --needed --noconfirm $needed_packages
    fi
else
    echo "Unsupported distribution. Please manually install: $needed_packages"
fi

# Auto-detect DVD path if not provided
if [ -z "$dvd_file_path" ]; then
    dvd_device=$(lsblk -ln -o NAME,TYPE | grep rom | awk '{print $1}' | head -1)
    [ -n "$dvd_device" ] && dvd_file_path="/dev/$dvd_device"
    echo "DVD file path detected: $dvd_file_path"
fi

# Check if valid path
if [ ! -e "$dvd_file_path" ]; then
    echo "Error: DVD path not found at $dvd_file_path"
    exit 1
fi

# Mount DVD if needed
mount_point=$(lsblk -no MOUNTPOINT "$dvd_file_path" | head -1)
if [ -z "$mount_point" ]; then
    echo "DVD is not mounted, mounting now..."
    mount_point="/media/dvd"

    # Ensure mount point exists
    if [ ! -d "$mount_point" ]; then
        echo "Creating mount point $mount_point"
        sudo mkdir -p "$mount_point"
    fi
    sudo mount "$dvd_file_path" "$mount_point" || { echo "Failed to mount DVD"; exit 1; }
fi
echo "DVD is mounted at $mount_point"

# Get movie name
read -p 'Movie Name: ' dname

# Create output directory
output_dir="./outs/$dname"
mkdir -p "$output_dir"

# Get disk info
echo "Getting disk info"
./venv/bin/python main.py --scan -i "$dvd_file_path" > "./outs/$current_timestamp.nfo" 2>/dev/null
echo "Disk info saved at ./outs/$current_timestamp.nfo"

# Rip DVD
echo "Ripping DVD"
rip_dir="./outs/$current_timestamp"
./venv/bin/python main.py -i "$dvd_file_path" -o "$rip_dir"
echo "DVD ripped into $rip_dir"

# Unmount DVD
echo "Unmounting DVD"
sudo umount "$dvd_file_path" && echo "DVD unmounted successfully"

# Server details
read -p "Server IP: " servip
read -sp "Server password: " serverpass
echo ""  # New line after password input

# Check if server info is provided
if [ -z "$servip" ]; then
    echo "Warning: No server IP provided, skipping upload"
    upload=false
else
    upload=true
fi

# Process files
echo "Processing ripped files"
dnum=1
for f in "$rip_dir"/*; do
    if [ -f "$f" ]; then
        filename="$dname - $dnum.mp4"
        target_path="$output_dir/$filename"

        echo "Processing: $f -> $target_path"
        mv -f "$f" "$target_path"

        # Upload file if server info available
        if $upload; then
            echo "Uploading $filename to $servip..."

            # Using sshpass with rsync for efficient transfer
            if sshpass -p "$serverpass" rsync -avz --progress "$target_path" "jellyfin@$servip:/mrd/media/movies/$dname/" 2>/dev/null; then
                echo "Successfully uploaded $filename"
            else
                echo "Failed to upload $filename"
            fi
        fi

        dnum=$((dnum+1))
    fi
done

echo "DONE!"
exit 0
