#!/bin/bash

# This script uses solana gossip to get the full list of validators,
# measures latency, checks if port 8899 is open, and outputs the results.

PATH=/home/ubuntu/.local/share/solana/install/active_release/bin:/home/ubuntu/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin

cd /home/ubuntu/scripts

# Store the version and gossip output in variables for reuse
VERSION=$(solana -V | cut -d" " -f2)
GOSSIP=$(solana gossip)
cat /dev/null > nc.log

date > validator_list.txt
echo "Version: $VERSION" >> validator_list.txt

# Parse the gossip output
for ip in $(echo "$GOSSIP" | grep -v none | grep $VERSION | awk -F'|' '/8899/{print $1}')
do
   key=$(echo "$GOSSIP" | grep $ip | awk -F'|' '{print $2}')

   # Measure latency
   latency=$(ping -c 5 -W 5 $ip | tail -1 | awk -F'/' '{print $5}')

   # Check if port 8899 is open
   nc -zv -w 5 $ip 8899 >> nc.log
   port_open=$?

   # Prepare the output format
   echo "$key, $ip, ${latency}ms, $(if [ $port_open -eq 0 ]; then echo open; else echo 0; fi)" >> validator_list.txt
done

grep "open" validator_list.txt | sort -t, -k3,3n | awk -F ', ' '$3 ~ /^[0-9.]+ms$/ {print $1}' | tr -d " " | head -5 > nearest_validators.txt
