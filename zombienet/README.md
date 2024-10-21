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

## How to update chainspecs

In case the relaychain version should be updated to reflect what is deployed on production, a new chainspec must be generated and added to this folder to replace the current one.

### Spiritnet/Polkadot

1. Head to the [polkadot-fellows/runtimes repo](https://github.com/polkadot-fellows/runtimes)
2. Checkout the right tag corresponding to the new version to deploy in the Zombienet environment, e.g., [`v1.3.3`](https://github.com/polkadot-fellows/runtimes/tree/v1.3.3)
3. Run `cargo run --release --features fast-runtime -p chain-spec-generator -- polkadot-local > out.json`, which saves the new chainspec into a temporary `out.json` file
4. Move the file into `runtimes/spiritnet` and rename it to `polkadot-local-fast-<version_tag>-<commit_sha>.json`, e.g., `polkadot-local-fast-v1.3.3-55bd514`
5. Update the `spiritnet/.env` file to set the right Docker image tag for the `RELAY_IMAGE` variable

### Peregrine/Paseo

1. Head to the [paseo-network/runtimes repo](https://github.com/paseo-network/runtimes)
2. Checkout the right tag corresponding to the new version to deploy in the Zombienet environment, e.g., [`v1.3.1`](https://github.com/paseo-network/runtimes/tree/v1.3.1)
3. Run `cargo run --release --features fast-runtime -p chain-spec-generator -- paseo-local > out.json`, which saves the new chainspec into a temporary `out.json` file
4. Move the file into `runtimes/peregrine` and rename it to `paseo-local-fast-<version_tag>-<commit_sha>.json`, e.g., `polkadot-local-fast-v1.3.1-e1fd37c`
5. Update the `peregrine/.env` file to set the right Docker image tag for the `RELAY_IMAGE` variable
