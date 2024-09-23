# KILT Zombienet utilities

This project contains setup code to spawn [Zombienet](https://github.com/paritytech/zombienet)-based deployments for both Spiritnet and Peregrine runtimes.

## Requirements

The Zombienet config assumes the presence of a properly configured `kubectl` cluster.
One local cluster can be installed with [minikube](https://minikube.sigs.k8s.io/docs/start/?arch=%2Fmacos%2Farm64%2Fstable%2Fbinary+download).
For other options, please refer to the Kubernetes documentation.

## How to spawn

First, `cd` into the `zombienet` folder and run first `nvm use` to configure the right Node version, and then `yarn` to install any Node dependencies.

Both Spiritnet and Peregrine deployments rely on the following env variables:

* `RELAY_IMAGE`: The Docker image for relaychain nodes.
* `RELAY_RPC`: The RPC port to expose for relaychain Alice node.
* `PARA_IMAGE`: The Docker image for KILT nodes.
* `PARA_RPC`: The RPC port to expose for KILT Alice node.

By default, both Spiritnet and Peregrine folders contain a `.env` file which uses the following defaults:

```sh
RELAY_IMAGE=parity/polkadot:v1.15.2
RELAY_RPC=50001
PARA_IMAGE=kiltprotocol/kilt-node:1.14.3
PARA_RPC=50010
```

To spin up a deployment using the defaults provided in the relative `.env` file, run either `yarn spawn:peregrine:with-env` or `yarn spawn:spiritnet:with-env`.

If values are explicitly set in the env, then run either `yarn spawn:peregrine` or `yarn spawn:spiritnet`, which will assume those values have already been set elsewhere.

After spawning the network, you can connect to the relaychain Alice node on `ws://localhost:<RELAY_RPC>` and to the KILT Alice node on `ws://localhost:<PARA_RPC>`.
