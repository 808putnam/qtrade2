#!/bin/bash

# Display help menu
usage() {
    echo ""
    echo "tf-lat-example.sh - help"
    echo "==============================================================================="
    echo  ""
    echo "General script to setup appropriate environment and run a terrafrom command for"
    echo "Latitude.sh infrastructure."
    echo ""
    echo "-------------------------------------------------------------------------------"
    echo "Options"
    echo "-------------------------------------------------------------------------------"
    echo "--help                               Displays help menu"
    echo "--cmd=<option>                       Specify the terraform command to run."
    echo "                                     init"
    echo "                                     validate"
    echo "                                     plan"
    echo "                                     apply"
    echo "                                     destroy"
    echo "                                     Shortcut: -c"
    echo ""
    echo ""
    echo "-------------------------------------------------------------------------------"
    echo ""

    exit
}

# Parse input arguments
while [[ $# -gt 0 ]]
do
key="$1"

case $key in
    --help)
    usage
    ;;
    --reference)
    reference
    ;;
    --cmd=*|-c)
    if [[ "$key" == "-c" ]]; then
        CMD="$2"
        shift 2
    else
        CMD="${key#*=}"
        shift
    fi
    ;;
    *)
    echo "Unknown option: $key"
    usage
    ;;
esac
done

# Validate input arguments
if [[ "$CMD" != "init"  && \
      "$CMD" != "validate"  && \
      "$CMD" != "plan"   && \
      "$CMD" != "apply"     && \
      "$CMD" != "destroy" ]]; then
    echo "Invalid --cmd: $CMD"
    usage
fi

# Amazon S3 backend configuration for terraform
export AWS_ACCESS_KEY_ID=$TERRAFORM_AWS_ACCESS_KEY_ID
export AWS_REGION=$TERRAFORM_AWS_REGION
export AWS_SECRET_ACCESS_KEY=$TERRAFORM_AWS_SECRET_ACCESS_KEY

echo "AWS Identity for terraform actions:"
aws sts get-caller-identity

# Latitude.sh auth. key to be picked up by terrform latitude.sh provider
export LATITUDESH_AUTH_TOKEN=$QTRADE_LATITUDESH_AUTH_TOKEN

# Run terraform command
terraform $CMD