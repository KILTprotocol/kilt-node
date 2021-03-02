![](https://user-images.githubusercontent.com/1248214/57789522-600fcc00-7739-11e9-86d9-73d7032f40fc.png) <img src="https://www.parity.io/assets/img/built-on-substrate-badge.svg" height=180/>

[![Build and Test](https://github.com/KILTprotocol/mashnet-node/workflows/Build%20and%20Test/badge.svg)](https://github.com/KILTprotocol/mashnet-node/actions)

# KILT mashnet-node (previously prototype-chain)

The KILT blockchain nodes use Parity Substrate as the underlying blockchain
technology stack extended with our DID, CType, Attestation and hierarchical Trust Modules.
Substrate Documentation:

[Substrate Tutorials](https://substrate.dev/en/tutorials)

[Substrate JSON-RPC API](https://polkadot.js.org/docs/substrate/rpc)

[Substrate Reference Rust Docs](https://substrate.dev/rustdocs/v2.0.0/sc_service/index.html)

- [KILT mashnet-node (previously prototype-chain)](#kilt-mashnet-node-previously-prototype-chain)
  - [How to use TL;DR](#how-to-use-tldr)
  - [How to use](#how-to-use)
    - [Images / Building](#images--building)
      - [Dockerhub](#dockerhub)
      - [Building docker image](#building-docker-image)
      - [Build code without docker](#build-code-without-docker)
      - [Building](#building)

## How to use TL;DR

Start chain and connect to alice bootnode:

```
docker run -p 9944:9944 kiltprotocol/mashnet-node --chain kilt-testnet
```

Start dev chain (producing blocks without requirements) for local development with all WebSocket Interfaces and Remote Procedure Calls enabled and specified WebSockets RPC server TCP port. Default would be only Local Procedure Calls enabled:

```
docker run -p 9944:9944 kiltprotocol/mashnet-node --dev --ws-port 9944 --ws-external --rpc-external --rpc-methods=unsafe
```

## How to use

To start a node, you need to use an existing image from [DockerHub](https://hub.docker.com/r/kiltprotocol/mashnet-node), or build the code yourself and decide on the command to execute.

### Images / Building

To build the code, or get a prebuilt image, you have these options:

- Docker image from [DockerHub](https://hub.docker.com/r/kiltprotocol/mashnet-node)
- Building the docker image yourself
- Building the code without docker

#### Dockerhub

To get the image from dockerhub execute following command:

```
docker pull kiltprotocol/mashnet-node
```

and run the node by executing:

```
docker run -p 9944:9944 kiltprotocol/mashnet-node [node command]
```

#### Building docker image

Clone this repo and navigate into it.

Build docker image:

```
docker build --build-arg NODE_TYPE=mashnet-node -t local/mashnet-node .
```

start, by running:

```
docker run -p 9944:9944 local/mashnet-node [node command]
```

For execution see the section about [Commands](#commands).

#### Build code without docker

You need to have [rust and cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed. Clone this repo and navigate into it.

You can build it by executing these commands:

```
./scripts/init.sh
```

#### Building

```
# debug builds
cargo build -p mashnet-node

# release builds
cargo build --release -p mashnet-node

# build the parachain
cargo build --release -p kilt-parachain
```

start, by running:

```
cargo run --release -p mashnet-node -- [node command]
```

The node should be connected to the KILT testnet.