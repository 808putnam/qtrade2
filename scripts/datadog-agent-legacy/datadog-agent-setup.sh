cd /tmp

# Clean up any previous installations
if [ -f install_script_agent7.sh ]; then
    rm install_script_agent7.sh
fi

# Run the install which we expect to fail at the end
curl -LOS https://install.datadoghq.com/scripts/install_script_agent7.sh
chmod +x install_script_agent7.sh
if ! ./install_script_agent7.sh; then
    echo "Warning: install_script_agent7.sh failed, but continuing with the script."
fi

# Build out the contents of datadog.yaml
if [ -f datadog-config-dynamic ]; then
    rm datadog-config-dynamic
fi
echo "api_key: $DD_API_KEY"   > datadog-config-dynamic
echo "hostname: $(hostname)" >> datadog-config-dynamic
cat datadog-config-dynamic /workspaces/qtrade/scripts/datadog/datadog-config-base /workspaces/qtrade/scripts/datadog/datadog.yaml.example > /etc/datadog-agent/datadog.yaml.tmp
mv /etc/datadog-agent/datadog.yaml.tmp /etc/datadog-agent/datadog.yaml

# Add in after workspace setup task 
# Setup logging
# mkdir -p /etc/datadog-agent/conf.d/qtrade.d 
# cp /workspaces/qtrade/scripts/datadog/qtrade.d-conf.yaml /etc/datadog-agent/conf.d/qtrade.d/conf.yaml

# Start the agent
service datadog-agent restart
