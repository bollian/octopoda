#!/bin/sh

# exit if a command fails
set -e

# check to make sure the user provided a block device
BOOT_DISK="$1"
if [ ! -b "$BOOT_DISK" ]; then
    echo "Please provide a disk to format (must be a block device)." >&2
    exit 1
fi

# create the master boot record and a single partition for
# the firmware and kernel files
sfdisk "$BOOT_DISK" << PARTITION_SCHEME
label: mbr

size=512MiB,name=boot,type=c
PARTITION_SCHEME

mkfs.fat -F 32 "$BOOT_DISK"p1
