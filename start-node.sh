#!/bin/bash

# start-node.sh - A script to start a node within the KILT test network

##### Constants

ALICE_BOOT_NODE_HASH=Qmf9Vcxjf5woQP9Znv7xnahCLbA6vXFm8PfnqWcDGgr4Ve
BOB_BOOT_NODE_HASH=QmTKngF1X4Zawh5Zi5sUq6F6o1NQbTPFnrXY8QpfpnkstH
CHARLIE_BOOT_NODE_HASH=QmW2U3aNywuG9S2z16HYKJWe83LDSxsiaavBummT2dJh8J
TELEMETRY_URL=ws://telemetry-backend.kilt-prototype.tk:1024

##### Functions

lookup_boot_node() {
    node=$1
    boot_node_domain="bootnode-${node}.kilt-prototype.tk"
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
  -p, --purge-userdata              Purges all chain-dependend user data in auxiliary services (ctypes, contacts, messages, ...). 
                                    Needs SERVICES_SECRET environment variable.

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
purge_userdata=0
dry_run=0
rpc=0
validator=0

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
        -p | --purge-userdata ) purge_userdata=1
                                ;;
        -d | --dry-run )        dry_run=1
                                ;;
        -r | --rpc )            rpc=1
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

# NODE_KEY = The hex-encoded ed25519 key used for libp2p networking.
if [[ ! -z "$NODE_KEY" ]]; then
    arg_node_key=" --node-key ${NODE_KEY}"
fi

# NODE_SEED = The seed for the validator account to be used. Has to be combined with NODE_KEY
if [[ "$validator" = "1" ]]; then
    arg_validator=" --key ${NODE_SEED} --validator"
    echo "Starting KILT validator node"
else
    echo "Starting KILT full node"
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

if [[ "$purge_userdata" = "1" ]]; then
    echo "Purging user data in services (SERVICES_SECRET=${SERVICES_SECRET})..."

    curl -X DELETE -H "Authorization: ${SERVICES_SECRET}" https://services.kilt.io/ctype
    curl -X DELETE -H "Authorization: ${SERVICES_SECRET}" https://services.kilt.io/messaging
    curl -X DELETE -H "Authorization: ${SERVICES_SECRET}" https://services.kilt.io/contacts
fi

if [[ "$rpc" = "1" ]]; then
    arg_rpc=" --ws-port 9944 --ws-external --rpc-external"
fi

command="./target/release/node --chain ./chainspec.json --port 30333${arg_rpc}${arg_validator}${arg_node_key}${arg_boot_node_connect}${arg_node_name}${arg_telemetry}"

if [[ "$dry_run" = "1" ]]; then
    echo "Dry run."
    echo "Command: $command"
    exit 0
fi

echo "Running: $command"
`${command}`
