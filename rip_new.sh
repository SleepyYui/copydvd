#!/bin/bash

# get dvd file path
dvd_file_path=$1
current_timestamp=$(date +%s)

needed_packages="handbrake-cli python3 vlc sshpass rsync mediainfo"

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

# Set default NAS config file
nas_config="$HOME/.copydvd_nas.conf"

# Load NAS config if exists
if [ -f "$nas_config" ]; then
    source "$nas_config"
    echo "Loaded NAS configuration from $nas_config"
else
    # Create default NAS config
    echo "# Configure your NAS settings" > "$nas_config"
    echo "NAS_IP=\"\"" >> "$nas_config"
    echo "NAS_USER=\"jellyfin\"" >> "$nas_config"
    echo "NAS_PASS=\"\"" >> "$nas_config"
    echo "NAS_MOVIE_PATH=\"/mrd/media/movies\"" >> "$nas_config"
    echo "AUTO_UPLOAD=false" >> "$nas_config"
    echo "Created default NAS configuration at $nas_config"
    echo "Please edit it with your NAS details for automatic uploads"
    source "$nas_config"
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

# Try to automatically detect movie name from DVD
echo "Trying to detect movie title automatically..."
detected_title=""

# Method 1: Try to read from DVD label
dvd_label=$(blkid -o value -s LABEL "$dvd_file_path" 2>/dev/null)
if [ -n "$dvd_label" ] && [ "$dvd_label" != "CDROM" ]; then
    detected_title=$dvd_label
fi

# Method 2: Try to find VIDEO_TS.IFO file and extract title
if [ -z "$detected_title" ] && [ -f "$mount_point/VIDEO_TS/VIDEO_TS.IFO" ]; then
    # Use strings to extract readable text and look for potential title
    possible_title=$(strings "$mount_point/VIDEO_TS/VIDEO_TS.IFO" | grep -E '^[A-Z0-9 _-]{5,50}$' | head -1)
    if [ -n "$possible_title" ]; then
        detected_title=$possible_title
    fi
fi

# Get movie name with detected title as default
if [ -n "$detected_title" ]; then
    read -p "Movie Name [$detected_title]: " input_name
    dname=${input_name:-$detected_title}
else
    read -p 'Movie Name: ' dname
fi

# Auto-detect year if possible
detected_year=""
if [[ "$dname" =~ .*[^0-9](19[0-9]{2}|20[0-9]{2})[^0-9].* ]]; then
    detected_year=$(echo "$dname" | grep -oE '(19|20)[0-9]{2}' | head -1)
elif [ -f "$mount_point/VIDEO_TS/VIDEO_TS.IFO" ]; then
    # Try to extract year from DVD info
    possible_year=$(strings "$mount_point/VIDEO_TS/VIDEO_TS.IFO" | grep -oE '(19|20)[0-9]{2}' | head -1)
    if [ -n "$possible_year" ]; then
        detected_year=$possible_year
    fi
fi

# Get movie year with detected year as default
if [ -n "$detected_year" ]; then
    read -p "Movie Year [$detected_year]: " input_year
    movie_year=${input_year:-$detected_year}
else
    read -p 'Movie Year (optional): ' movie_year
fi

# Format final movie name
if [ -n "$movie_year" ]; then
    formatted_name="$dname ($movie_year)"
else
    formatted_name="$dname"
fi

# Create output directory
output_dir="./outs/$formatted_name"
mkdir -p "$output_dir"

# Get disk info
echo "Getting disk info"
./venv/bin/python main.py --scan -i "$dvd_file_path" > "$output_dir/movie_info.nfo" 2>/dev/null
echo "Disk info saved at $output_dir/movie_info.nfo"

# Rip DVD
echo "Ripping DVD"
rip_dir="./outs/$current_timestamp"
./venv/bin/python main.py -i "$dvd_file_path" -o "$rip_dir"
echo "DVD ripped into $rip_dir"

# Unmount DVD
echo "Unmounting DVD"
sudo umount "$dvd_file_path" && echo "DVD unmounted successfully"

# Check if NAS config needs to be set or updated
if [ -z "$NAS_IP" ] || [ "$AUTO_UPLOAD" = "false" ]; then
    read -p "Configure NAS now? [y/n]: " configure_nas
    if [[ "$configure_nas" =~ ^[Yy](es)?$ ]]; then
        read -p "NAS IP: " NAS_IP
        read -p "NAS Username [jellyfin]: " input_user
        NAS_USER=${input_user:-jellyfin}
        read -sp "NAS Password: " NAS_PASS
        echo ""
        read -p "NAS Movie Path [/mrd/media/movies]: " input_path
        NAS_MOVIE_PATH=${input_path:-/mrd/media/movies}
        read -p "Enable automatic uploads? [y/n]: " auto_upload_choice
        if [[ "$auto_upload_choice" =~ ^[Yy](es)?$ ]]; then
            AUTO_UPLOAD=true
        else
            AUTO_UPLOAD=false
        fi
        
        # Update config file
        echo "# Configure your NAS settings" > "$nas_config"
        echo "NAS_IP=\"$NAS_IP\"" >> "$nas_config"
        echo "NAS_USER=\"$NAS_USER\"" >> "$nas_config" 
        echo "NAS_PASS=\"$NAS_PASS\"" >> "$nas_config"
        echo "NAS_MOVIE_PATH=\"$NAS_MOVIE_PATH\"" >> "$nas_config"
        echo "AUTO_UPLOAD=$AUTO_UPLOAD" >> "$nas_config"
        echo "NAS configuration saved to $nas_config"
    fi
fi

# Process files
echo "Processing ripped files"
# Count total files for better renaming
total_files=$(find "$rip_dir" -type f | wc -l)
dnum=1

# Determine naming format based on number of files
if [ "$total_files" -eq 1 ]; then
    # Single file - just use movie name
    naming_format="single"
else
    # Multiple files - ask for type
    echo "Multiple files detected. Choose naming format:"
    echo "1. Movie parts (Part 1, Part 2, etc.)"
    echo "2. Disc numbers (Disc 1, Disc 2, etc.)"
    echo "3. Episodes (Ep01, Ep02, etc.)"
    read -p "Select format [1]: " format_choice
    case "$format_choice" in
        2) naming_format="disc" ;;
        3) naming_format="episode" ;;
        *) naming_format="part" ;;
    esac
fi

for f in "$rip_dir"/*; do
    if [ -f "$f" ]; then
        # Determine file suffix based on naming format and total files
        case "$naming_format" in
            "single")
                filename="$formatted_name.mp4"
                ;;
            "part")
                if [ "$total_files" -gt 1 ]; then
                    filename="$formatted_name - Part $dnum.mp4"
                else
                    filename="$formatted_name.mp4"
                fi
                ;;
            "disc")
                filename="$formatted_name - Disc $dnum.mp4"
                ;;
            "episode")
                # Format episode number with leading zero if needed
                if [ "$total_files" -gt 9 ]; then
                    ep_num=$(printf "%02d" $dnum)
                else
                    ep_num=$dnum
                fi
                filename="$formatted_name - Ep$ep_num.mp4"
                ;;
        esac

        target_path="$output_dir/$filename"
        echo "Processing: $f -> $target_path"
        mv -f "$f" "$target_path"

        # Upload file if NAS is configured and auto-upload is enabled
        if [ -n "$NAS_IP" ] && [ "$AUTO_UPLOAD" = "true" ]; then
            echo "Uploading $filename to $NAS_IP..."
            
            # Create remote directory if it doesn't exist
            remote_dir="$NAS_MOVIE_PATH/$formatted_name"
            sshpass -p "$NAS_PASS" ssh "$NAS_USER@$NAS_IP" "mkdir -p \"$remote_dir\"" 2>/dev/null
            
            # Using sshpass with rsync for efficient transfer
            if sshpass -p "$NAS_PASS" rsync -avz --progress "$target_path" "$NAS_USER@$NAS_IP:$remote_dir/" 2>/dev/null; then
                echo "Successfully uploaded $filename"
            else
                echo "Failed to upload $filename"
                
                # Retry with basic scp as fallback
                echo "Retrying with scp..."
                if sshpass -p "$NAS_PASS" scp "$target_path" "$NAS_USER@$NAS_IP:$remote_dir/" 2>/dev/null; then
                    echo "Successfully uploaded $filename using scp"
                else
                    echo "Failed to upload $filename. Please check NAS configuration."
                fi
            fi
        elif [ -n "$NAS_IP" ]; then
            echo "NAS configured but automatic upload disabled. To upload manually:"
            echo "scp \"$target_path\" $NAS_USER@$NAS_IP:$NAS_MOVIE_PATH/$formatted_name/"
        fi

        dnum=$((dnum+1))
    fi
done

echo "DONE!"
exit 0
