#!/bin/bash

# Simple monitoring script for Agave Validator
# Add this to crontab to run every 5 minutes:
# */5 * * * * /home/ubuntu/scripts/monitor-validator.sh

# Check if the validator is running
if ! pgrep -f "agave-validator" > /dev/null; then
    echo "[$(date)] ERROR: Validator is not running!" | tee -a /home/ubuntu/Solana/log/validator-monitor.log

    # Attempt to restart the service
    sudo systemctl restart agave-validator

    # Optional: Send notification via email or other means
    # mail -s "ALERT: Agave Validator not running" your@email.com <<< "The validator stopped running and an automatic restart was attempted."

    exit 1
fi

# Check if the validator is catching up (you might need to adjust this part)
# This uses the Solana CLI to check validator status
VALIDATOR_STATUS=$(agave-cli catchup /home/ubuntu/Solana/validator-keypair.json --url http://localhost:8899 2>&1)
BEHIND=$(echo "$VALIDATOR_STATUS" | grep -o "behind by [0-9]* slots" | grep -o "[0-9]*")

if [[ -n "$BEHIND" && "$BEHIND" -gt 100 ]]; then
    echo "[$(date)] WARNING: Validator is behind by $BEHIND slots!" | tee -a /home/ubuntu/Solana/log/validator-monitor.log

    # Optional: Send notification if falling significantly behind
    # mail -s "WARNING: Agave Validator falling behind" your@email.com <<< "The validator is behind by $BEHIND slots."
fi

# Log successful check
echo "[$(date)] INFO: Validator check OK" >> /home/ubuntu/Solana/log/validator-monitor.log

exit 0
