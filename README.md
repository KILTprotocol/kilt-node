# prototype-chain

Substrate node implementation for the KILT prototype

## Running a local node that connects to KILT prototype testnet in AWS

There are master boot nodes running in the KILT testnet:

* Alice (bootnode-alice.kilt-prototype.tk)
* Bob (bootnode-bob.kilt-prototype.tk)

To start a node and connect to Alice you can use the shell script `start-node.sh`:

```
./start-node.sh --account-name Charly --connect-to Alice
``` 

You can use any of the accounts declared in the chain spec to connect (Alice, Bob, Charly, Dave, Eve, Ferdie).

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
docker run 348099934012.dkr.ecr.eu-central-1.amazonaws.com/kilt/prototype-chain ./start-node.sh --account-name Charly --connect-to Alice
```
The node should be connected to the KILT testnet.


  b. For local development with an isolated local chain, execute: 

```
docker run -p 9944:9944 348099934012.dkr.ecr.eu-central-1.amazonaws.com/kilt/prototype-chain ./start-node.sh --account-name Alice
```
