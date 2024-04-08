#!/usr/bin/fish

set -l options (fish_opt -s d -l dev --required-val)
set options $options (fish_opt -s h -l help)

argparse $options -- $argv

if set -q _flag_help
    echo "Haalp!"
    exit 0
end

if set -q _flag_dev
    set -l UEFI (cargo build --release &| grep UEFI | awk -F ': ' '{print($4)}')
    if test -e $UEFI
        echo "Copying $UEFI to $_flag_dev"
        sudo dd if=$UEFI of=$_flag_dev && sync
    else
        echo "Error getting UEFI img path"
    end

    exit 0
end

echo "Usage: ./run.fish <DISK>"
