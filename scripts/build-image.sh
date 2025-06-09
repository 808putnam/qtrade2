#!/bin/bash
set -e          # Exit immediately if any command fails
set -u          # Exit if trying to use an undefined variable (catches typos)
set -o pipefail # Exit if any command in a pipeline fails (not just the last one)

# Initialize variables with default values
BREAK_SYSTEM_PACKAGES=""
IMAGE=""
PUBLISH="none"
UBUNTU_VERSION=""
TAG=""
# Initialize environment variables with fallback to empty string if not set
# This prevents "unbound variable" errors from set -u when env vars aren't defined
CR_USER="${CR_USER:-}"
CR_PAT="${CR_PAT:-}"

# Display help menu
usage() {
    echo ""
    echo "build-image.sh"
    echo "==============================================================================="
    echo  ""
    echo "General docker build script used to build and optionally publish an image."
    echo ""
    echo "-------------------------------------------------------------------------------"
    echo "IMPORTANT"
    echo "-------------------------------------------------------------------------------"
    echo "If you create a new dev version, then update docker/Dockerfile.prod to"
    echo "reference it."
    echo ""
    echo "-------------------------------------------------------------------------------"
    echo "Setup"
    echo "-------------------------------------------------------------------------------"
    echo "1. This script expects to be run from the scripts folder."
    echo "2. For publishing to GitHub Container Registry, the following environment variables"
    echo "   must be set:"
    echo "   CR_USER: The GitHub username"
    echo "    CR_PAT: The GitHub personal access token with read:packages, write:packages and"
    echo "            delete:packages scope set."
    echo ""
    echo "-------------------------------------------------------------------------------"
    echo "Options"
    echo "-------------------------------------------------------------------------------"
    echo "--help              Displays help menu"
    echo ""
    echo "--image=<option>    Specify the image to build."
    echo "                       client - qtrade-client:          production image for qtrade-client"
    echo "                       dev    - qtrade-dev:             codespace for development"
    echo ""
    echo "--ubuntu=<option>   Specify the ubuntu OS version"
    echo "                        --image=client: This option is hard-coded to 20.04"
    echo "                        --image=dev:    This option is mandatory"
    echo "                        20.04"
    echo "                        24.04"
    echo ""
    echo "--publish=<option>  Optionally publish build to GitHub or local registry."
    echo "                       Defaults to none."
    echo "                       github"
    echo "                       local"
    echo "                       none (default)"
    echo ""
    echo "--tag=<option>      Specify the tag for the build (e.g., 1.0.0)."
    echo "                       This option is mandatory if you are publishing to GitHub."
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
    -i|--image=*)
    IMAGE="${i#*=}"
    shift
    ;;
    -u|--ubuntu=*)
    UBUNTU_VERSION="${i#*=}"
    shift
    ;;
    -p|--publish=*)
    PUBLISH="${i#*=}"
    shift
    ;;
    -t|--tag=*)
    TAG="${i#*=}"
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
if [[ "$IMAGE" != "client" && "$IMAGE" != "dev" ]]; then
    echo "Invalid -i|--image: $IMAGE"
    usage
fi
# Ubuntu version is mandatory when you building dev image and hard-coded to 20.04 for client
if [[ "$IMAGE" == "dev" && "$UBUNTU_VERSION" == "" ]]; then
    echo "--ubuntu is mandatory when --image is set to dev."
    usage
fi
if [[ "$IMAGE" == "client" ]]; then
    UBUNTU_VERSION="20.04"
fi
if [[ "$UBUNTU_VERSION" != "20.04" && "$UBUNTU_VERSION" != "24.04" ]]; then
    echo "Invalid -u|--ubuntu: $UBUNTU_VERSION"
    usage
fi
# Set PEP 668 flag for Ubuntu 24.04+
if [[ "$UBUNTU_VERSION" == "24.04" ]]; then
    BREAK_SYSTEM_PACKAGES="--break-system-packages"
else
    BREAK_SYSTEM_PACKAGES=""
fi
if [[ "$PUBLISH" != "github" && "$PUBLISH" != "local" && "$PUBLISH" != "none" ]]; then
    echo "Invalid -p|--publish: $PUBLISH"
    usage
fi
# Tag is mandatory when publishing to GitHub
if [[ "$PUBLISH" == "github" && "$TAG" == "" ]]; then
    echo "--tag is mandatory when --publish is set to github."
    usage
fi
if [[ "$PUBLISH" == "github" && "$CR_USER" == "" ]]; then
    echo "The environment variable CR_USER must be set to a GitHub user"
    echo "when publishing to GitHub."
    usage
fi
if [[ "$PUBLISH" == "github" && "$CR_PAT" == "" ]]; then
    echo "The environment variable CR_PAT must be set to a GitHub personal access token"
    echo "with read:packages, write:packages and delete:packages scope set when"
    echo "publishing to GitHub."
    usage
fi

# Determine Docker image name and Dockerfile path for commands below
if [[ "$IMAGE" == "dev" ]]; then
    DOCKERFILE_PATH="./docker/Dockerfile.dev"
    IMAGE_NAME="qtrade-dev-$UBUNTU_VERSION"
fi
if [[ "$IMAGE" == "client" ]]; then
    DOCKERFILE_PATH="./docker/Dockerfile.client"
    IMAGE_NAME="qtrade-client"
fi

# IMPORTANT: We are changing the build context to the root folder of the repo
#            so we can access additional build artifacts to copy to the resulting
#            image. This requires us to pass in the --file flag to specify our Dockerfile.
#
# Reference: https://docs.docker.com/engine/reference/commandline/build/#text-files
cd ..

# Go for the build
if [[ "$TAG" == "" ]]; then
     docker build -t $IMAGE_NAME \
     --progress=plain \
     --build-arg UBUNTU_VERSION=$UBUNTU_VERSION \
     --build-arg BREAK_SYSTEM_PACKAGES="$BREAK_SYSTEM_PACKAGES" \
     --file $DOCKERFILE_PATH .
     echo "Built: $IMAGE_NAME"
else
    docker build -t $IMAGE_NAME:$TAG \
    --progress=plain \
    --build-arg UBUNTU_VERSION=$UBUNTU_VERSION \
    --build-arg BREAK_SYSTEM_PACKAGES="$BREAK_SYSTEM_PACKAGES" \
    --file $DOCKERFILE_PATH .
    echo "Built: $IMAGE_NAME:$TAG"
fi

# Do we want to publish image also?
# Note that when publishing locally, you will need a loccal registry typically setup as follows:
# docker run -d \
# -p 5000:5000 \
# --restart-always \
# --name  registry \
# registry:2
# Reference: https://docs.docker.com/registry/deploying/#start-the-registry-automatically
if [[ "$PUBLISH" == "github" ]]; then
    echo $CR_PAT | docker login ghcr.io -u $CR_USER --password-stdin
    docker image tag $IMAGE_NAME:$TAG ghcr.io/808putnam/$IMAGE_NAME:$TAG
    docker push ghcr.io/808putnam/$IMAGE_NAME:$TAG
    echo "Pushed: $IMAGE_NAME:$TAG to GitHub"
elif [[ "$PUBLISH" == "local" ]]; then
    if [[ "$TAG" == "" ]]; then
        docker image tag $IMAGE_NAME localhost:5000/$IMAGE_NAME
        docker push localhost:5000/$IMAGE_NAME
        echo "Pushed: $IMAGE_NAME to local registry localhost:5000"
    else
        docker image tag $IMAGE_NAME:$TAG localhost:5000/$IMAGE_NAME:$TAG
        docker push localhost:5000/$IMAGE_NAME:$TAG
        echo "Pushed: $IMAGE_NAME:$TAG to local registry localhost:5000"
    fi
fi