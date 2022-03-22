#!/usr/bin/env bash

set -euo pipefail
set -o xtrace

source .env

sudo mkdir -p "/mnt/w1"
user="$(id -un)"
group="$(id -gn)"
sudo chown -R $user:$group /mnt/w1

IFS=','

read -ra addresses <<< "$BUB_SLOT_ADDRESSES"

for address in "${addresses[@]}"; do
    mkdir -p "/mnt/w1/$address"
    touch "/mnt/w1/$address/id"
done

address="$BUB_TEMP_ADDRESS"

mkdir -p "/mnt/w1/$address"
echo 4 | tee "/mnt/w1/$address/temperature12" >/dev/null
