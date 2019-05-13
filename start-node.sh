#!/bin/bash

# start-node.sh - A script to start a node within the KILT test network

##### Constants

CHAIN_NAME="kilt-testnet"
ALICE_BOOT_NODE_KEY=0000000000000000000000000000000000000000000000000000000000000001
ALICE_BOOT_NODE_KEY_HASH=QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN
BOB_BOOT_NODE_KEY=0000000000000000000000000000000000000000000000000000000000000002
BOB_BOOT_NODE_KEY_HASH=QmXiB3jqqn2rpiKU7k1h7NJYeBg8WNSx9DiTRKz9ti2KSK
CHARLIE_BOOT_NODE_KEY=0000000000000000000000000000000000000000000000000000000000000003
CHARLIE_BOOT_NODE_KEY_HASH=QmYcHeEWuqtr6Gb5EbK7zEhnaCm5p6vA2kWcVjFKbhApaC
TELEMETRY_URL=ws://telemetry-backend.kilt-prototype.tk:1024

##### Functions

lookup_boot_node() {
    node=$1
    boot_node_domain="bootnode-${node}.kilt-prototype.tk"
    echo "Performing lookup for boot node ${boot_node_domain}"
    if [[ "$node" = "Alice" ]]; then
        alice_boot_node_ip=`dig ${boot_node_domain} A +short`
        boot_node_ipfs="$boot_node_ipfs /ip4/${alice_boot_node_ip}/tcp/30333/p2p/${ALICE_BOOT_NODE_KEY_HASH}"
    elif [[ "$node" = "Bob" ]]; then
        bob_boot_node_ip=`dig ${boot_node_domain} A +short`
        boot_node_ipfs="$boot_node_ipfs /ip4/${bob_boot_node_ip}/tcp/30333/p2p/${BOB_BOOT_NODE_KEY_HASH}"
    elif [[ "$node" = "Charlie" ]]; then
        charlie_boot_node_ip=`dig ${boot_node_domain} A +short`
        boot_node_ipfs="$boot_node_ipfs /ip4/${charlie_boot_node_ip}/tcp/30333/p2p/${CHARLIE_BOOT_NODE_KEY_HASH}"
    fi
}



usage()
{
cat <<HELP_USAGE
Usage:
  $0  -a <account-name> [...]

  If you want to start a boot node, just use "Alice" or "Bob" as account name.

  -a, --account-name ACCOUNT_NAME   The name of the account to start the node with (Alice | Bob | Charlie).
  -n, --node-name NODE_NAME    The arbitrary name of the node (e.g. "charly-node-1234")
  -c, --connect-to BOOT_NODE_NAME  The names of the boot nodes to connect to, separated with a comma ("alice" | "bob" | "Charlie")
  -d, --dry-run Flag indicating to only show the resulting command instead of executing it
  -t, --telemetry Flag indicating whether or not to send data to the telemetry server
  -p, --purge-userdata Purges all chain-dependend user data in auxiliary services (ctypes, contacts, messages, ...)
  -r, --rpc Whether to activate rpc

  Examples:

  Start Alice (boot node) and purge all user data in services components:
  ./start-node.sh -a Alice -p

  Start Bob (boot node) that connects to Alice:
  ./start-node.sh -a Bob -c Alice

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

while [[ "$1" != "" ]]; do
    case $1 in
        -a | --account-name )   shift
                                account_name=$1
                                ;;
        -n | --node-name )      shift
                                node_name=$1
                                ;;
        -c | --connect-to )     shift
                                bootnodes=$1
                                ;;
        -t | --telemetry )      telemetry=1
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
arg_account_name=
arg_rpc=

if [[ "$account_name" = "Alice" ]]; then
    arg_node_key=" --node-key ${ALICE_BOOT_NODE_KEY}"
elif [[ "$account_name" = "Bob" ]]; then
    arg_node_key=" --node-key ${BOB_BOOT_NODE_KEY}"
elif [[ "$account_name" = "Charlie" ]]; then
    arg_node_key=" --node-key ${CHARLIE_BOOT_NODE_KEY}"
fi

if [[ ! -z "$account_name" ]]; then
    arg_account_name=" --key //${account_name} --validator"
    echo "Starting KILT validator node with account '${account_name}'"
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

command="./target/debug/node --chain ${CHAIN_NAME} --port 30333${arg_rpc}${arg_account_name}${arg_node_key}${arg_boot_node_connect}${arg_node_name}${arg_telemetry}"

if [[ "$dry_run" = "1" ]]; then
    echo "Dry run."
    echo "Command: $command"
    exit 0
fi

echo "Running: $command"
`${command}`
