#!/bin/bash

#
# This script starts a kilt node and connects to the master boot node of the KILT testnet.
#
# Run:
# ./kilt-node-testnet.sh --key Charly --name "CHARLY"
#

ALICE_BOOT_NODE_DOMAIN=bootnode-alice.kilt-prototype.tk
ALICE_BOOT_NODE_IP=`dig $ALICE_BOOT_NODE_DOMAIN A +short`
ALICE_BOOT_NODE_KEY=QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN
ALICE_BOOT_NODE_IPFS=/ip4/$ALICE_BOOT_NODE_IP/tcp/30333/p2p/$ALICE_BOOT_NODE_KEY


#BOB_BOOT_NODE_DOMAIN=bootnode-bob.kilt-prototype.tk
#BOB_BOOT_NODE_IP=`dig $BOB_BOOT_NODE_DOMAIN A +short`
#BOB_BOOT_NODE_KEY=QmXiB3jqqn2rpiKU7k1h7NJYeBg8WNSx9DiTRKz9ti2KSK
#BOB_BOOT_NODE_IPFS=/ip4/$BOB_BOOT_NODE_IP/tcp/30333/p2p/$BOB_BOOT_NODE_KEY

MASTER_BOOT_NODE_IPFS=`./lookup-master-bootnode-testnet.sh`

# Connect to master boot node
echo "Master boot node: $MASTER_BOOT_NODE_IPFS..."
./target/debug/node --chain kilt-testnet --bootnodes $MASTER_BOOT_NODE_IPFS --port 30333 --ws-port 9944 --validator --ws-external --rpc-external --telemetry-url ws://telemetry-backend.kilt-prototype.tk:1024 "$@"

