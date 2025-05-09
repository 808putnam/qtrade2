#!/usr/bin/env bash

ARGS=$*

echo "starting qtrade-client: $ARGS"
trap 'jobs -p | xargs kill' SIGINT SIGTERM EXIT

cd /app/qtrade-client/ || exit 1

# start qtrade-client
echo "starting client..."
/app/qtrade-client/qtrade-client $ARGS &

wait -n
exit $?
