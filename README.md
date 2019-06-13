![](https://user-images.githubusercontent.com/1248214/57789522-600fcc00-7739-11e9-86d9-73d7032f40fc.png)

# KILT mashnet-node (previously prototype-chain)

Substrate node implementation for the KILT prototype

## How to use
To start a node, you need to build the code, or use an existing image and decide for a command to execute.

## Building / Images
To build the code, or get a prebuilt image, you have these options:
- Docker image from dockerhub
- Building the docker image yourself
- Building the code without docker

### Dockerhub
To get the image from dockerhub execute following command:
```
docker pull kiltprotocol/mashnet-node
```
and run the node by executing:
```
docker run -p 9944:9944 kiltprotocol/mashnet-node [node command]
```

### Building docker image
Clone this repo and navigate into it.

Build docker image:
```
docker build -t local/mashnet-node .
```
start, by running:
```
docker run -p 9944:9944 local/mashnet-node [node command]
```

### Build code without docker
You need to have rust and cargo installed and configured properly.

You can build it by executing these commands:
```
./init.sh
./build.sh
cargo build
```

For execution see the section about commands.

## Commands
To start the node you have following options:
- start-node.sh helper script
- executing the node binary directly
### Helper script
We include a helper script, which sets up the arguments used for the node binary.

Use it by executing:
```
./start-node.sh --help
```

This can be used in all building strategies:
```
docker run -p 9944:9944 kiltprotocol/mashnet-node ./start-node.sh --connect-to alice
```
or
```
docker run -p 9944:9944 local/mashnet-node ./start-node.sh --connect-to alice
```
or if you build it without docker:
```
./start-node.sh --connect-to alice
```

### Node binary
The node binary, which gets build lies in the directory
```
./target/debug/node [arguments]
```

If you want to start a local dev-chain you can execute:
```
./target/debug/node --dev
```

If you are using a docker image, run:
```
docker run -p 9944:9944 kiltprotocol/mashnet-node ./target/debug/node --dev --ws-port 9944 --ws-external --rpc-external
```


## Examples

### Running a local node that connects to KILT prototype testnet in AWS

There are master boot nodes running in the KILT testnet:

* Alice (bootnode-alice.kilt-prototype.tk)
* Bob (bootnode-bob.kilt-prototype.tk)

To start a node and connect to alice you can use the shell script `start-node.sh`:

```
./start-node.sh --connect-to alice
``` 

If you want to connect to this node via RPC, add the `--rpc` flag:
```
./start-node.sh --connect-to alice --rpc
```

Run `./start-node.sh --help` for more information.

### Running a node with local image, which runs a dev-chain
build docker image (only do if code has changed, takes ~15 min)
```
docker build -t dev/mashnet-node .
```
run chain in dev mode locally
```
docker run -p 9944:9944 dev/mashnet-node ./target/debug/node --dev --ws-port 9944 --ws-external --rpc-external
```

## Development with AWS images:

Make sure to have the `awscli` installed. Otherwise Install it via `brew install awscli` (Mac).
You also need to have your docker daemon system running (on mac, just download and install the docker application).

1. Login to Amazon ECR

```
 $(aws ecr get-login --no-include-email --region eu-central-1)
```

2. Pull the latest image from Amazon ECR

```
docker pull 348099934012.dkr.ecr.eu-central-1.amazonaws.com/kilt/prototype-chain:latest
```

3. Run node

To run a node and connect it to the KILT testnet: Run the image and pass the command to start a node:

```
docker run 348099934012.dkr.ecr.eu-central-1.amazonaws.com/kilt/prototype-chain ./start-node.sh --connect-to alice
```
The node should be connected to the KILT testnet.

## Updating with latest substrate-node-template

The command `substrate-node-new`, described in https://docs.substrate.dev/docs/creating-a-custom-substrate-chain downloads a node-template, which this repo bases on.
We just added our modules to the runtime.

To update it, a stable template can be copied from https://github.com/shawntabrizi/substrate-package.
Just copy the contents of substrate-node-template and add our changes on top.

If the mentioned repo of shawntabrizi isn't updated anymore, the `substrate-node-new` command can still be used to get a fresh node-template. It might need some changes to work, though.
