#!/bin/bash

# Notes:
# 1. The "datadog" ansible playbook will take care of normally copying the file
#    to the correct location.
# 2. This script is easing live development/debugging on the bare-metal instance.
# 3. IMPORTANT: This script is not under source code control - so if you want to
#               keep changes, you need to copy them to the repo manually.
sudo cp ../files/qtrade-custom-checks.service      /etc/systemd/system/qtrade-custom-checks.service
sudo cp ../files/otel-collector-config-testnet.yml /etc/otelcol-contrib/otel-collector-config-testnet.yml
