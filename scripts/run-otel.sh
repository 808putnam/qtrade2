#!/bin/bash

# Display help menu
usage() {
    echo ""
    echo "run-otel.sh"
    echo "==============================================================================="
    echo  ""
    echo "Run the otel/opentelemetry-collector container."
    echo ""
    echo "-------------------------------------------------------------------------------"
    echo "Setup"
    echo "-------------------------------------------------------------------------------"
    echo "1. This script expects to be run from the scripts folder so that it can find the"
    echo "   appropriate otel config file."
    echo "2. Open Telemetry config files are maintained at:"
    echo "   infra/otel"
    echo ""
    echo "-------------------------------------------------------------------------------"
    echo "Options"
    echo "-------------------------------------------------------------------------------"
    echo "--help              Displays help menu"
    echo ""
    echo "--config=<option>   Specify the config file to use."
    echo "                       local    - otel-collector-config-localnet.yaml"
    echo "-------------------------------------------------------------------------------"
    echo ""

    exit
}

# Parse input arguments
for i in "$@"
do
case $i in
    -h|--help)
    usage
    shift
    ;;
    -c|--config=*)
    CONFIG="${i#*=}"
    shift
    ;;
    *)
    echo "Unknown option: $i"
    usage
    shift
    ;;
esac
done

# Validate input arguments and set defaults
if [[ "$CONFIG" != "local" ]]; then
    echo "Invalid -c|--config: $CONFIG"
    usage
fi

# Run the otel/opentelemetry-collector container interactively
# with the designated config file
if [[ "$CONFIG" == "local" ]]; then
    docker run --rm -it \
        --name otel-collector \
        -p 4317:4317 \
        -p 4318:4318 \
        -v /workspaces/qtrade/infra/otel:/cfg \
        otel/opentelemetry-collector:0.121.0 \
        --config=/cfg/otel-collector-config-localnet.yaml
fi