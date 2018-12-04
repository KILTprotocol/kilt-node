# prototype-chain

Substrate node implementation for the KILT prototype

## Running a local node that connects to the KILT prototype testnet in AWS

There is a master boot node running in the KILT testnet:

* bootnode-alice (bootnode-alice.kilt-prototype.tk)

To connect to the Alice node you can use the shell script `kilt-node-testnet.sh`:

```
cd scripts/
kilt-node-testnet.sh --key Charly --name "CHARLY"
``` 

You can use any of the accounts declared in the chain spec to connect (Alice, Bob, Charly, Dave, Eve, Ferdie).


### Running a node inside a docker container

Make sure to have the `awscli` installed. Otherwise Install it via `brew install awscli` (Mac).
You also need to have your docker daemon system running (on mac, just download the docker application).

Login to Amazon ECR

```
 $(aws ecr get-login --no-include-email --region eu-central-1)
```

Pull the latest image from Amazon ECR

```
docker pull 348099934012.dkr.ecr.eu-central-1.amazonaws.com/kilt/prototype-chain:latest
```

Run the image and pass the command to start a node

```
docker run 348099934012.dkr.ecr.eu-central-1.amazonaws.com/kilt/prototype-chain ./kilt-node-testnet.sh --key Charly --name "CHARLY"
```

The node should be connected to the KILT testnet.