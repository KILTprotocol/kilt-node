#!/bin/bash

# start-node.sh - A script to start a node within the KILT test network

##### Constants

# Testnet
ALICE_BOOT_NODE_HASH=Qmf9Vcxjf5woQP9Znv7xnahCLbA6vXFm8PfnqWcDGgr4Ve
BOB_BOOT_NODE_HASH=QmTKngF1X4Zawh5Zi5sUq6F6o1NQbTPFnrXY8QpfpnkstH
CHARLIE_BOOT_NODE_HASH=QmW2U3aNywuG9S2z16HYKJWe83LDSxsiaavBummT2dJh8J
# Devnet
ALICE_DEVNET_HASH=QmR627o6Sj2smbqh3XSuP11YzThuBz3QTDz2QMYwB9oo8U
BOB_DEVNET_HASH=QmVFbWDS3fDK3bV3p9AmSSjEk2vKTCAYU4jEizLX8mMMzp
CHARLIE_DEVNET_HASH=QmQWsRDKfD5tKyL9U442nPmxpW4Xdnv9uCtano4meay4nv

BASE_URL=kilt-prototype.tk
TELEMETRY_URL=ws://telemetry-backend.kilt-prototype.tk:1024

HEX_AURA=61757261
HEX_GRAN=6772616e

##### Functions

lookup_boot_node() {
	node=$1
	boot_node_domain="bootnode-${node}.${BASE_URL}"
	echo "Performing lookup for boot node ${boot_node_domain}"
	if [[ "$node" = "alice" ]]; then
		alice_boot_node_ip=`dig ${boot_node_domain} A +short`
		boot_node_ipfs="$boot_node_ipfs /ip4/${alice_boot_node_ip}/tcp/30333/p2p/${ALICE_BOOT_NODE_HASH}"
	elif [[ "$node" = "bob" ]]; then
		bob_boot_node_ip=`dig ${boot_node_domain} A +short`
		boot_node_ipfs="$boot_node_ipfs /ip4/${bob_boot_node_ip}/tcp/30333/p2p/${BOB_BOOT_NODE_HASH}"
	elif [[ "$node" = "charlie" ]]; then
		charlie_boot_node_ip=`dig ${boot_node_domain} A +short`
		boot_node_ipfs="$boot_node_ipfs /ip4/${charlie_boot_node_ip}/tcp/30333/p2p/${CHARLIE_BOOT_NODE_HASH}"
	fi
}



usage()
{
cat <<HELP_USAGE
Usage:
  $0 [...]

  If you want to start a full node, you just have to provide "alice", "bob" and/or "charlie" to connect to the boot nodes.

  -c, --connect-to BOOT_NODE_NAME   The names of the boot nodes to connect to, separated with a comma
									["alice" | "bob" | "charlie"]
  -n, --node-name NODE_NAME         The arbitrary name of the node (e.g. "charly-node-1234")
  -d, --dry-run                     Flag indicating to only show the resulting command instead of executing it
  -t, --telemetry                   Flag indicating whether or not to send data to the telemetry server
  -r, --rpc                         Whether to activate rpc
  -v, --validator                   Whether the node should be a validator. Needs NODE_SEED and NODE_KEY environment variables.
  --devnet                          Use the KILT devnet instead of the testnet

  Examples:

  Start full node
  ./start-node.sh

  Start full node that connects to Alice boot node:
  ./start-node.sh -c Alice

  Start full node that connects to Alice and Bob and exposes an rpc endpoint:
  ./start-node.sh -c Alice,Bob -n charly-node-123 --rpc
HELP_USAGE
}

##### Main


bootnodes=
node_name=
account_name=
telemetry=0
dry_run=0
rpc=0
validator=0
devnet=0

while [[ "$1" != "" ]]; do
	case $1 in
		-n | --node-name )      shift
								node_name=$1
								;;
		-c | --connect-to )     shift
								bootnodes=$1
								;;
		-t | --telemetry )      telemetry=1
								;;
		-v | --validator )      validator=1
								;;
		-d | --dry-run )        dry_run=1
								;;
		-r | --rpc )            rpc=1
								;;
		--devnet )              devnet=1
								;;
		-h | --help )           usage
								exit
								;;
		* )                     usage
								exit 1
	esac
	shift
done


arg_boot_node_connect=
arg_node_key=
arg_node_name=
arg_telemetry=
arg_validator=
arg_rpc=
arg_chain=" --chain ./chainspec.json"
arg_base_path=" --base-path db"

# NODE_KEY = The hex-encoded ed25519 key used for libp2p networking.
if [[ ! -z "$NODE_KEY" ]]; then
	arg_node_key=" --node-key ${NODE_KEY}"
fi

# NODE_SEED = The seed for the validator account to be used. Has to be combined with NODE_KEY
if [[ "$validator" = "1" ]]; then
	arg_validator=" --validator --keystore-path keystore"
	mkdir -p keystore
	echo "\"0x$AUTH_SEED\"" > keystore/$HEX_GRAN$AUTH_PUB_KEY
	echo "\"0x$AUTH_SEED\"" > keystore/$HEX_AURA$AUTH_PUB_KEY
	echo "Starting KILT validator node"
else
	echo "Starting KILT full node"
fi

if [[ "$devnet" = "1" ]]; then
	arg_chain=" --chain kilt-devnet"
	BASE_URL="devnet.kilt.io"
	ALICE_BOOT_NODE_HASH=${ALICE_DEVNET_HASH}
	BOB_BOOT_NODE_HASH=${BOB_DEVNET_HASH}
	CHARLIE_BOOT_NODE_HASH=${CHARLIE_DEVNET_HASH}
fi

if [[ ! -z "$bootnodes" ]]; then
	boot_node_ipfs=
	echo "Trying to connect to boot node(s) '$bootnodes'..."
	for i in $(echo $bootnodes | tr "," "\n")
	do
		lookup_boot_node $i
	done
	if [[ -z "$boot_node_ipfs" ]]; then
		echo "Boot node address lookup failed for boot nodes '$bootnodes'"
		exit 1
	else
		echo "Boot-node IPFS locations: $boot_node_ipfs"
		arg_boot_node_connect=" --bootnodes${boot_node_ipfs}"
	fi
fi

if [[ ! -z "$node_name" ]]; then
	random_suffix=`cat /dev/urandom | env LC_CTYPE=C tr -cd 'a-f0-9' | head -c 5`
	node_name="${node_name}-${random_suffix}"
	arg_node_name=" --name ${node_name}"
fi

if [[ "$telemetry" = "1" ]]; then
	arg_telemetry=" --telemetry-url ${TELEMETRY_URL}"
fi

if [[ "$rpc" = "1" ]]; then
	arg_rpc=" --ws-port 9944 --ws-external --rpc-external"
fi

command="./target/release/mashnet-node --port 30333${arg_chain}${arg_rpc}${arg_validator}${arg_node_key}${arg_boot_node_connect}${arg_node_name}${arg_telemetry}${arg_base_path}"

if [[ "$dry_run" = "1" ]]; then
	echo "Dry run."
	echo "Command: $command"
	exit 0
fi

echo "Running: $command"
`${command}`
