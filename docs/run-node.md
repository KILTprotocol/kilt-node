# How to run a node

There are several ways to run the node locally.
In case you haven't made changes to the code, we **recommend to use [our docker images](https://hub.docker.com/r/kiltprotocol/mashnet-node)** which will save you quite some time by not being required to compile for ~10-30 minutes.

## KILT runtimes

Please note that we have two different runtimes
* `mashnet-node` - The standalone "validator" node runtime, mainly for development purposes
* `kilt-parachain` - The parachain-compliant "collator" node runtime which runs on Rococo and will be required by all future KILT collators

Throughout this document, you can swap `mashnet-node` with `kilt-parachain` and vice versa.
For development, we recommend to run the `mashnet-node`.

## How to run a node (TLDR)

To start a node, you need to use an existing image from [DockerHub](https://hub.docker.com/r/kiltprotocol/mashnet-node), or [build the code yourself](#option-2-build-code-without-docker) and decide on the [command](#node-commands) to execute.

### Use a pre-compiled docker image from DockerHub

```bash
docker pull kiltprotocol/mashnet-node
```
### Run the docker image

When you start a dev chain, it produces blocks without any requirements like additional validators.
For local development, we recommend to use the below node options to have 
* WebSocket Interface enabled + WebSockets specified 
* Remote Procedure Calls enabled + RPC server specified
* TPC server specified
because per default, only Local Procedure Calls would be enabled 

```bash
docker run -p 9944:9944 kiltprotocol/mashnet-node --dev --ws-port 9944 --ws-external --rpc-external --rpc-methods=unsafe
```

Here's how you would start the chain in general and connect to the Alice, Bob and Charlie bootnodes:

```bash
docker run -p 9944:9944 kiltprotocol/mashnet-node [node command]
```

For execution see the sections about [Node Commands](#node-commands) and our [recommendations](#recommendation-for-development).

## How to build and run a node (more detailed)

To build the code, or get a prebuilt image, you have the following two options:

1. [Building the docker image yourself](#option-1-build-a-docker-image-yourself)
2. [Building the code without docker](#option-2-build-code-without-docker)

### Option 1: Build a docker image yourself

1. Clone this repo and navigate into it.
2. Build docker image:

```
docker build --build-arg NODE_TYPE=mashnet-node -t local/mashnet-node .
```

3. Start the image by running:

```bash
docker run -p 9944:9944 local/mashnet-node [node command]
```

### Option 2: Build code without docker

You need to have [rust and cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.
The following steps expect you to have cloned our repo and navigated into it.

#### Setup Rust

Building requires the WASM toolchain and is most reliable with a specific rust version, we recommend to set this by executing [our init script](../scripts/init.sh): 

```bash
./scripts/init.sh
```

For more information about the required setup for Substrate, have a look at the [installation guide of the Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/), especially if you are using Windows.

#### Build command

There are two ways of building your node: `debug` or `release`.
For development purposes, the `debug` mode suffices and also compiles faster.
For production, you should always run in `release` mode.

```bash
# build in debug mode
cargo build -p $RUNTIME

# build in release mode 
cargo build --release -p $RUNTIME

# build the standalone mashnet node
cargo build --release -p mashnet-node

# build the kilt-parachain collator node 
cargo build --release -p kilt-parachain
```

#### Start command

```bash
cargo run --release -p $RUNTIME -- [node command]
```

Start a dev chain with detailed logging

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/$RUNTIME -lruntime=debug --dev
```

## Node commands

After the node has been build, refer to the embedded documentation to learn more about the capabilities and configuration parameters that it exposes:

```bash
./target/release/$RUNTIME --help
```

### Recommendation for development

For developing purposes we usually use the following:

```bash
--tmp --dev --ws-port 9944 --port 30444 --alice --ws-external --rpc-external --rpc-cors all --rpc-methods=unsafe
```

Heres an overview of what these commands stand for:

```
--tmp
    Run a temporary node.

    A temporary directory will be created to store the configuration and will be deleted at the end of the
    process.

    Note: the directory is random per process execution. This directory is used as base path which includes:
    database, node key and keystore.

--dev
     Specify the development chain. This enables block authoring when running a single node.

--ws-port <PORT>
    Specify WebSockets RPC server TCP port. This is the port you use when connecting via the Polkadot JS Apps.

--port <PORT>
    Specify p2p protocol TCP port

--alice
    Shortcut for `--name Alice --validator` with session keys for `Alice` added to keystore

--rpc-external
     Listen to all RPC interfaces.

     Default is local. Note: not all RPC methods are safe to be exposed publicly. Use an RPC proxy server to
     filter out dangerous methods. More details: <https://github.com/paritytech/substrate/wiki/Public-RPC>. Use
     `--unsafe-rpc-external` to suppress the warning if you understand the risks.

--rpc-cors <ORIGINS>
    Specify browser Origins allowed to access the HTTP & WS RPC servers.

    A comma-separated list of origins (protocol://domain or special `null` value). Value of `all` will disable
    origin validation. Default is to allow localhost and <https://polkadot.js.org> origins. When running in
    --dev mode the default is to allow all origins.
    --rpc-methods <METHOD SET>
    RPC methods to expose.

    - `Unsafe`: Exposes every RPC method.
    - `Safe`: Exposes only a safe subset of RPC methods, denying unsafe RPC methods.
    - `Auto`: Acts as `Safe` if RPC is served externally, e.g. when `--{rpc,ws}-external` is passed,
        otherwise acts as `Unsafe`. [default: Auto]  [possible values: Auto, Safe, Unsafe]

--rpc-methods <METHOD SET>
    RPC methods to expose.

    - `Unsafe`: Exposes every RPC method.
    - `Safe`: Exposes only a safe subset of RPC methods, denying unsafe RPC methods.
    - `Auto`: Acts as `Safe` if RPC is served externally, e.g. when `--{rpc,ws}-external` is passed,
        otherwise acts as `Unsafe`. [default: Auto]  [possible values: Auto, Safe, Unsafe]
```

### More helpful commands

```
-d, --base-path <PATH>
    Specify custom base path

--bootnodes <ADDR>...
    Specify a list of bootnodes

--chain <CHAIN_SPEC>
    Specify the chain specification.

    It can be one of the predefined ones (dev, local, or staging) or it can be a path to a file with the
    chainspec (such as one exported by the `build-spec` subcommand).


--rpc-port <PORT>
    Specify HTTP RPC server TCP port


-l, --log <LOG_PATTERN>...
    Sets a custom logging filter. Syntax is <target>=<level>, e.g. -lsync=debug.

    Log levels (least to most verbose) are error, warn, info, debug, and trace. By default, all targets log
    `info`. The global log level can be set with -l<level>.
```
