#!/bin/bash

# Set the minimum required disk size in terabytes (3.5TB)
MIN_SIZE=3

# Find all disks using lsblk, exclude the OS disk (nvme0n1), and filter by size
disks=$(lsblk -d -n -o NAME,SIZE | grep -vE 'loop|sr|sda' | awk -v min_size=$MIN_SIZE '$2 ~ /T$/ && substr($2, 1, length($2)-1) >= min_size {print $1}')

# Convert to an array and check if we have at least 2 suitable disks
IFS=$'\n' read -r -d '' -a disk_array <<< "$disks"

if [ "${#disk_array[@]}" -lt 2 ]; then
   echo "Error: Less than 2 disks with at least 3.5TB of space were found."
   exit 1
fi

# Disk mount points
MOUNT_POINT1="/home/ubuntu/Solana/ledger"
MOUNT_POINT2="/home/ubuntu/Solana/solana-accounts"

# Create the directories if they do not exist
mkdir -p "$MOUNT_POINT1"
mkdir -p "$MOUNT_POINT2"

# Function to format, partition, and mount the disk
partition_and_mount_disk() {
   local disk=$1
   local mount_point=$2

   echo "Processing /dev/$disk..."

   # Create a GPT partition table and a single primary partition
   sudo parted -s /dev/$disk mklabel gpt
   sudo parted -s /dev/$disk mkpart primary ext4 0% 100%

   # Create the filesystem on the partition
   if [[ "$disk" == nvme* ]]; then
       sudo mkfs.ext4 /dev/${disk}p1
       sudo mount /dev/${disk}p1 "$mount_point"
       echo "/dev/${disk}p1 $mount_point ext4 defaults 0 2" | sudo tee -a /etc/fstab
   else
       sudo mkfs.ext4 /dev/${disk}1
       sudo mount /dev/${disk}1 "$mount_point"
       echo "/dev/${disk}1 $mount_point ext4 defaults 0 2" | sudo tee -a /etc/fstab
   fi
}

# Process the first disk
partition_and_mount_disk "${disk_array[0]}" "$MOUNT_POINT1"

# Process the second disk
partition_and_mount_disk "${disk_array[1]}" "$MOUNT_POINT2"

# Change ownership of the mounted directories
sudo chown -R ubuntu:ubuntu /home/ubuntu/Solana/*

echo "Testing the change using: sudo mount -a"
sudo mount -a

echo "Disks have been successfully partitioned, formatted, and mounted."
df -h
