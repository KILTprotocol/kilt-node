#!/bin/bash

#
# This script starts a kilt master boot node with provided account, name and node-key
#
# Run:
# ./kilt-master-bootnode-testnet.sh --key Alice --name "ALICE" --node-key 0000000000000000000000000000000000000000000000000000000000000001
#

echo "Starting KILT master boot node..."
./target/debug/node --chain kilt-testnet --validator --port 30333 --ws-port 9944 --ws-external --rpc-external --telemetry-url ws://telemetry-backend.kilt-prototype.tk:1024 "$@"

