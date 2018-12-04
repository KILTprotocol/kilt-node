#!/bin/bash

#
# This script performs a lookup for the master boot node and prints out its ipfs address.
#
# Run:
# ./lookup-master-bootnode-testnet.sh
# Prints out the IPFS of the master boot node:
# e.g.:
#   /ip4/3.120.148.48/tcp/30333/p2p/QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN
#

ALICE_BOOT_NODE_DOMAIN=bootnode-alice.kilt-prototype.tk
ALICE_BOOT_NODE_IP=`dig $ALICE_BOOT_NODE_DOMAIN A +short`
ALICE_BOOT_NODE_KEY=QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN
ALICE_BOOT_NODE_IPFS=/ip4/$ALICE_BOOT_NODE_IP/tcp/30333/p2p/$ALICE_BOOT_NODE_KEY

echo $ALICE_BOOT_NODE_IPFS