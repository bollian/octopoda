#!/usr/bin/env sh

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

BOOT_PARTITION=$(lsblk "$BOOT_DISK" --raw -o PATH | tail -n 1)

mkfs.fat -F 32 "$BOOT_PARTITION"
mkdir -p media
mount "$BOOT_PARTITION" media

cat > media/config.txt << EOF
arm_64bit=1
init_uart_clock=48000000
EOF
cp firmware/boot/bootcode.bin firmware/boot/fixup.dat firmware/boot/start.elf bin/kernel8.img media

umount media
rmdir media
