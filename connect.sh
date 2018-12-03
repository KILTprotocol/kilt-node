#!/bin/bash

ALICE_BOOT_NODE_DOMAIN=bootnode-alice.kilt-prototype.tk
ALICE_BOOT_NODE_IP=`dig $ALICE_BOOT_NODE_DOMAIN A +short`
ALICE_BOOT_NODE_KEY=QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN
ALICE_BOOT_NODE_IPFS=/ip4/$ALICE_BOOT_NODE_IP/tcp/30333/p2p/$ALICE_BOOT_NODE_KEY

BOB_BOOT_NODE_DOMAIN=bootnode-bob.kilt-prototype.tk
BOB_BOOT_NODE_IP=`dig $BOB_BOOT_NODE_DOMAIN A +short`
BOB_BOOT_NODE_KEY=QmXiB3jqqn2rpiKU7k1h7NJYeBg8WNSx9DiTRKz9ti2KSK
BOB_BOOT_NODE_IPFS=/ip4/$BOB_BOOT_NODE_IP/tcp/30333/p2p/$BOB_BOOT_NODE_KEY

echo "ALICE_BOOT_NODE_DOMAIN: $ALICE_BOOT_NODE_DOMAIN"
echo "ALICE_BOOT_NODE_IP: $ALICE_BOOT_NODE_IP"
echo "BOB_BOOT_NODE_DOMAIN: $BOB_BOOT_NODE_DOMAIN"
echo "BOB_BOOT_NODE_IP: $BOB_BOOT_NODE_IP"

# Connect to Alice
echo "Connecting to $ALICE_BOOT_NODE_IPFS..."
./target/debug/node --chain local --bootnodes $ALICE_BOOT_NODE_IPFS --port 30334 --validator "$@"

# Connect to Bob
# echo "Connecting to $BOB_BOOT_NODE_IPFS..."
# ./target/debug/node --chain local --bootnodes $BOB_BOOT_NODE_IPFS --port 30334 --validator "$@"

