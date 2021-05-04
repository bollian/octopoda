#!/bin/sh

# exit if a command fails
set -e

# check to make sure the user provided a directory
INSTALL_DIR="$1"
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Please provide a directory to install to" >&2
    exit 1
fi

# determine the root of where we're copying the files from, since
# this script may not be called from the menagerie project directory
MN_DIR=$(dirname "$0")

# copy all the necessary files over to the installation directory
cp "$MN_DIR"/firmware/boot/fixup.dat \
    "$MN_DIR"/firmware/boot/start.elf \
    "$MN_DIR"/firmware/boot/bootcode.bin \
    "$MN_DIR"/octopoda/bin/kernel8.img \
    "$INSTALL_DIR"

# helpful reminder
echo "Make sure there are no other files in the directory! Otherwise, you won't boot."
