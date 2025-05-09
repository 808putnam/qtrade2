#!/bin/bash

# Notes:
# 1. The "agave" ansible playbook will take care of normally copying the files
#    to the correct location.
# 2. This script is easing live development/debugging on the bare-metal instance.
# 3. IMPORTANT: The /home/ubuntu/scripts folder where this is deployed to is not under source code control - so if you want to
#               keep changes, you need to copy them to the repo manually.

sudo cp ../files/solana-validator /etc/logrotate.d/solana-validator
