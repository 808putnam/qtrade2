#!/bin/bash

# Display help menu
usage() {
    echo ""
    echo "stage-yellowstone.sh"
    echo "==============================================================================="
    echo  ""
    echo "Script to build and stage yellowstone-grpc library for development."
    echo ""
    echo "IMPORTANT: Run from scripts folder"
    echo ""
    echo "-------------------------------------------------------------------------------"
    echo "Options"
    echo "-------------------------------------------------------------------------------"
    echo "--help              Displays help menu"
    echo ""
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
    *)
    echo "Unknown option: $i"
    usage
    shift
    ;;
esac
done

cd ..
rm -rvf build && mkdir build

cd yellowstone-grpc/yellowstone-grpc-geyser
cargo build

cd ../..
cp yellowstone-grpc/target/debug/config-check                 build/config-check
cp yellowstone-grpc/target/debug/config-check.d               build/config-check.d
cp yellowstone-grpc/target/debug/libyellowstone_grpc_geyser.* build/
cp yellowstone-grpc/yellowstone-grpc-geyser/config.json       build/config-full.json


echo "yellowstone-grpc library staged for development in build folder"