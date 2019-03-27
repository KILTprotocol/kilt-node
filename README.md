# prototype-chain

Substrate node implementation for the KILT prototype

## Updating with latest substrate-node-template

The command `substrate-node-new`, described in https://docs.substrate.dev/docs/creating-a-custom-substrate-chain downloads a node-template, which this repo bases on.
We just added our modules to the runtime.

To update it, a stable template can be copied from https://github.com/shawntabrizi/substrate-package.
Just copy the contents of substrate-node-template and add our changes on top.

If the mentioned repo of shawntabrizi isn't updated anymore, the `substrate-node-new` command can still be used to get a fresh node-template. It might need some changes to work, though.

## Running a local node that connects to KILT prototype testnet in AWS

There are master boot nodes running in the KILT testnet:

* Alice (bootnode-alice.kilt-prototype.tk)
* Bob (bootnode-bob.kilt-prototype.tk)

To start a node and connect to Alice you can use the shell script `start-node.sh`:

```
./start-node.sh --connect-to Alice
``` 

If you want to connect to this node via RPC, add the `--rpc` flag:
```
./start-node.sh --connect-to Alice --rpc
```

Run `./start-node.sh --help` for more information.

### Running a node inside a docker container

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

  a. To run a node and connect it to the KILT testnet: Run the image and pass the command to start a node:

```
docker run 348099934012.dkr.ecr.eu-central-1.amazonaws.com/kilt/prototype-chain ./start-node.sh --connect-to Alice
```
The node should be connected to the KILT testnet.


  b. For local development with an isolated local chain, execute: 

```
# build docker image (only do if code has changed, takes ~15 min)
docker build -t prototype-chain .

# run chain in dev mode locally
docker run -p 9944:9944 prototype-chain ./target/debug/node --dev --ws-port 9944 --ws-external --rpc-external
```
