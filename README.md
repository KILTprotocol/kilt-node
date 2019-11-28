![](https://user-images.githubusercontent.com/1248214/57789522-600fcc00-7739-11e9-86d9-73d7032f40fc.png)

# KILT mashnet-node (previously prototype-chain)

The KILT blockchain nodes use Parity Substrate as the underlying blockchain
technology stack extended with our DID, CType, Attestation and hierarchical Trust Modules.
Substrate Documentation:

[High Level Docs](https://substrate.dev/docs/en/getting-started/)
[JSON-RPC](https://polkadot.js.org/api/substrate/rpc.html)
[Reference Rust Docs](https://substrate.dev/rustdocs/v1.0/substrate_service/index.html)

- [How to use TL;DR](#how-to-use-tl-dr)
- [How to use](#how-to-use)
  * [Images / Building](#images---building)
    + [Dockerhub](#dockerhub)
    + [Building docker image](#building-docker-image)
    + [Build code without docker](#build-code-without-docker)
    + [Building in dev mode](#building-in-dev-mode)
    + [Building in performant release mode](#building-in-performant-release-mode)
  * [Commands](#commands)
    + [Helper script](#helper-script)
    + [Node binary](#node-binary)
  * [Examples](#examples)
    + [Running a local node that connects to KILT prototype testnet in AWS](#running-a-local-node-that-connects-to-kilt-prototype-testnet-in-aws)
    + [Running a node with local image, which runs a dev-chain](#running-a-node-with-local-image--which-runs-a-dev-chain)
- [Development with AWS images](#development-with-aws-images)
- [Updating to latest substrate-node-template](#updating-to-latest-substrate-node-template)
- [Node Modules functionalities](#node-modules-functionalities)
  * [DID Module](#did-module)
    + [Add](#add)
    + [CRUD](#crud)
  * [CTYPE Module](#ctype-module)
  * [Attestation Module](#attestation-module)
    + [Add](#add)
    + [Revoke](#revoke)
    + [Lookup](#lookup)
  * [Hierarchy of Trust Module](#hierarchy-of-trust-module)
    + [Create root](#create-root)
    + [Add delegation](#add-delegation)
    + [Revoke](#revoke-1)

## How to use TL;DR
Start chain and connect to alice bootnode:
```
docker run -p 9944:9944 kiltprotocol/mashnet-node ./start-node.sh --connect-to alice
```

Start dev chain (producing blocks without requirements) for local development with all WebSocket Interfaces and Remote Procedure Calls enabled and specified WebSockets RPC server TCP port. Default would be only Local Procedure Calls enabled:
```
docker run -p 9944:9944 kiltprotocol/mashnet-node ./target/release/node --dev --ws-port 9944 --ws-external --rpc-external
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
docker build -t local/mashnet-node .
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
./scripts/build.sh
```
#### Building in dev mode
```
cargo build
```
start, by running:
```
./target/debug/node [node command]
```

#### Building in performant release mode
```
cargo build --release
```
start, by running:
```
./target/release/node [node command]
```

For execution see the section about commands.
### Commands
To start the node you have following options:
- start-node.sh helper script
- executing the node binary directly
#### Helper script
We include a helper script, which sets up the arguments used for the node binary.

Use it by executing:
```
./start-node.sh --help
```

This can be used in all building strategies (except dev mode built):
```
docker run -p 9944:9944 kiltprotocol/mashnet-node ./start-node.sh --connect-to alice
```
or
```
docker run -p 9944:9944 local/mashnet-node ./start-node.sh --connect-to alice
```
or if you built it without docker:
```
./start-node.sh --connect-to alice
```

#### Node binary
After finished building the node binary can be found in the `./target` directory:
for dev mode build:
```
./target/debug/node [node command]
```
for release mode build:
```
./target/release/node [node command]
```

If you want to start a local dev-chain after building in dev mode you can execute:
```
./target/debug/node --dev
```

If you are using a docker image, run:
```
docker run -p 9944:9944 kiltprotocol/mashnet-node ./target/release/node --dev --ws-port 9944 --ws-external --rpc-external
```


### Examples

#### Running a local node that connects to KILT prototype testnet in AWS

There are master boot nodes running in the KILT testnet:

* Alice (bootnode-alice.kilt-prototype.tk)
* Bob (bootnode-bob.kilt-prototype.tk)

To start a node and connect to alice you can use the shell script `start-node.sh`:

```
./start-node.sh --connect-to alice
``` 

If you want to connect to this node with all (default is local) WebSocket Interfaces and Remote Procedure Calls enabled and specified WebSockets RPC server TCP port
```
./start-node.sh --connect-to alice --rpc
```

Run `./start-node.sh --help` for more information.

#### Running a node with local image, which runs a dev-chain
build docker image (only do if code has changed, takes ~15 min)
```
docker build -t local/mashnet-node .
```
run chain in dev mode locally
```
docker run -p 9944:9944 local/mashnet-node ./target/release/node --dev --ws-port 9944 --ws-external --rpc-external
```

## Development with AWS images

Make sure to have the `awscli` installed. Otherwise, install it via `brew install awscli` (Mac).
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

## Updating to latest substrate-node-template

The command `substrate-node-new` (included in the substrate installation and described in https://substrate.dev/docs/en/tutorials/creating-your-first-substrate-chain) downloads a node-template, which this repo is based on.
We just added our modules to the runtime.

To update it, a stable template can be copied from https://github.com/shawntabrizi/substrate-package.
Just copy the contents of substrate-node-template and add our changes on top.

If the mentioned repo of shawntabrizi isn't updated anymore, the `substrate-node-new` command can still be used to get a fresh node-template. It might need some changes to work, though.

## Node Modules functionalities

The KILT blockchain is the heart and soul behind KILT protocol. It provides the immutable
transaction ledger for the various processes in the network.

`Building on the Parity Substrate Blockchain Framework`

During our first whiteboard phase, we were thinking about developing the KILT protocol on
Ethereum smart-contracts, but we realised that we would have less freedom of setting
transaction costs, while incurring a high level of overhead. Instead, we started our
development on [Parity Substrate](https://www.parity.io/substrate/), a general blockchain framework, and built up the KILT blockchain from scratch based on its module library.

Building our blockchain on Parity Substrate has multiple advantages. Substrate has a very
good fundamental [architecture](https://docs.substrate.dev/docs/architecture-of-a-runtime) and [codebase](https://github.com/paritytech/substrate) created by blockchain experts. Substrate
framework is developed in Rust, a memory efficient and fast compiled system programming
language, which provides a secure environment with virtually no runtime errors. Moreover, the
node runtime is also compiled to WebAssembly, so older version native nodes can always run
the latest version node runtime in a WebAssembly virtual machine to bypass the problem of a
blockchain fork. Importantly, there is a vibrant developer community and rich [documentation](https://docs.substrate.dev/).

Our implementation is based on the [substrate-node-template](https://github.com/rstormsf/substrate-node-template) library (skeleton template for
quickly building a substrate based blockchain), which is linked to the main Substrate
codebase.

__Remote Procedure Calls__

The Ethereum ecosystem highly leverages [JSON-RPC](https://www.jsonrpc.org/specification) where one can efficiently call methods and parameters directly on the blockchain node. Based on good experiences, developers
decided to use it in Substrate as well. The [Polkadot API](https://polkadot.js.org/api/) helps with communicating with the JSON-RPC endpoint, and the clients and services never have to talk directly with the endpoint.

__Blocktime__

The blocktime is currently set to 5 seconds, but this setting is subject to change based on
further research. We will consider what is affected by this parameter, and in the long term it
will be fine-tuned to a setting that provides the best performance and user experience for the
participants of the KILT network.

__Extrinsics and Block Storage__

In Substrate, the blockchain transactions are abstracted away and are generalised as
[extrinsics](https://docs.substrate.dev/docs/extrinsics) in the system. They are called extrinsics since they can represent any piece of information that is regarded as input from “the outside world” (i.e. from users of the network) to the blockchain logic. The blockchain transactions in KILT are implemented through these
general extrinsics, that are signed by the originator of the transaction. We use this framework
to write the KILT Protocol specific data entries on the Substrate based KILT blockchain: [DID],
[CTYPEhash], [Attestation] and [Delegation]. The processing of each of these entry types is
handled by our custom Substrate runtime node modules.

Under the current consensus algorithm, authority validator nodes (whose addresses are listed
in the genesis block) can create new blocks. These nodes [validate](https://docs.substrate.dev/docs/transaction-lifecycle-in-substrate) incoming transactions, put
them into the pool, and include them in a new block. While creating the block, the node
executes the transactions and stores the resulting state changes in its local storage. Note that
the size of the entry depends on the number of arguments the transaction, (i.e., the respective
extrinsic method) has. The size of the block is hence dynamic and will depend on the number
and type of transactions included in the new block. The valid new blocks are propagated
through the network and other nodes execute these blocks to update their local state (storage).

__Consensus Algorithm__

Since we are the only authority provider in the testnet phase, we use the simple [Aura](https://wiki.parity.io/Aura)
consensus mechanism. At a later stage, we most likely will change to [GRANDPA](https://github.com/paritytech/substrate#2-description), which
supposedly will be superior to Aura in many aspects. The consensus mechanism is also
subject to the future possibility to integrate the KILT network into the Polkadot ecosystem.

__Polkadot Integration__

As a further great advantage, by basing ourselves on Substrate we can easily connect to the
Polkadot ecosystem. This could provide security for the KILT network by leveraging the global
consensus in the Polkadot network. We are planning to integrate KILT into the [Polkadot](https://polkadot.network/)
network. It is fairly straightforward to achieve that by simply including specific Substrate
modules into the KILT Validator Node implementation. The exact details of this integration is
subject to future agreements between Polkadot and KILT and the technological development
of Polkadot, Substrate and KILT.

__KILT Tokens__

Coin transfer is implemented as a balance-based mechanism in Substrate. In our testnet,
every new identity gets 1000 KILT Tokens from a root entity in the system who is wired into
the genesis block. At a later stage in the testnet (Mash-Net) and the persistent testnet (WashNet), we are proposing to provide KILT Tokens for new developers wanting to join our network
on a simple request-provide based mechanism. Preferably, developers will be able to register
on our website, and we manually transfer KILT tokens to the registered developers.
Importantly, these test tokens will not be usable on our mainnet. After the launch of the mainnet
(Spirit-Net) and the public KILT Token sale, tokens will be available on cryptocurrency
exchanges.

### DID Module

The KILT blockchain node runtime defines an DID module exposing:
#### Add

```rust
add(origin, sign_key: T::PublicSigningKey, box_key: T::PublicBoxKey, doc_ref: Option<Vec<u8>>) -> Result
```

This function takes the following parameters:

- origin: public [ss58](https://wiki.parity.io/External-Address-Format-(SS58)) address of the caller of the method
- sign_key: the [ed25519](http://ed25519.cr.yp.to/) public signing key of the owner
- box_key: the [x25519-xsalsa20-poly1305](http://nacl.cr.yp.to/valid.html) public encryption key of the owner
- doc_ref: Optional u8 byte vector representing the reference (URL) to the DID
 document

The blockchain node verifies the transaction signature corresponding to the owner and
inserts it to the blockchain storage by using a map (done by the substrate framework):

```rust
T::AccountId => (T::PublicSigningKey, T::PublicBoxKey, Option<Vec<u8>>)
```
#### CRUD
As DID supports CRUD (Create, Read, Update, Delete) operations, a `get(dids)` method
reads a DID for an account address, the add function may also be used to update a DID and
a `remove(origin) -> Result` function that takes the owner as a single parameter removes the DID from the
map, so any later read operation call does not return the data of a removed DID.

### CTYPE Module

The KILT blockchain node runtime defines an CTYPE module exposing
```rust
add(origin, hash: T::Hash) -> Result
```
This function takes following parameters:

- origin: public [ss58](https://wiki.parity.io/External-Address-Format-(SS58)) address of the caller of the method
- hash: CTYPE hash as a [blake2b](https://blake2.net/) string

The blockchain node verifies the transaction signature corresponding to the creator and
inserts it to the blockchain storage by using a map (done by the substrate framework):

```rust
T::Hash => T::AccountId
```

### Attestation Module

The KILT blockchain node runtime defines an Attestation module exposing functions to
- add an attestation (`add`)
- revoke an attestation (`revoke`)
- lookup an attestation (`lookup`)
- lookup attestations for a delegation (used later in Complex Trust Structures)
on chain.

#### Add
```rust
add(origin, claim_hash: T::Hash, ctype_hash: T::Hash, delegation_id: Option<T::DelegationNodeId>) -> Result
```
The `add` function takes following parameters:
- origin: The caller of the method, i.e., public address ([ss58](https://wiki.parity.io/External-Address-Format-(SS58))) of the Attester
- claim_hash: The Claim hash as [blake2b](https://blake2.net/) string used as the key of the entry
- ctype_hash: The [blake2b](https://blake2.net/) hash of CTYPE used when creating the Claim
- delegation_id: Optional reference to a delegation which this attestation is based
on

The node verifies the transaction signature and insert it to the state, if the provided attester
didn’t already attest the provided claimHash. The attestation is stored by using a map:

```rust
T::Hash => (T::Hash,T::AccountId,Option<T::DelegationNodeId>,bool)
```

Delegated Attestations are stored in an additional map:

```rust
T::DelegationNodeId => Vec<T::Hash>
```

#### Revoke
```rust
revoke(origin, claim_hash: T::Hash) -> Result
```

The `revoke` function takes the claimHash (which is the key to lookup an attestation) as
argument. After looking up the attestation and checking invoker permissions, the revoked
flag is set to true and the updated attestation is stored on chain.

#### Lookup

The attestation lookup is performed with the `claimHash`, serving as the key to the
attestation store. The function `get_attestation(claimHash)` is exposed to the outside
clients and services on the blockchain for this purpose.

Similarly, as with the simple lookup, to query all attestations created by a certain delegate,
the runtime defines the function `get_delegated_attestations(DelegationNodeId)`
that is exposed to the outside.

### Hierarchy of Trust Module

The KILT blockchain node runtime defines a Delegation module exposing functions to
- create a root `create_root`
- add a delegation `add_delegation`
- revoke a delegation `revoke_delegation`
- revoke a whole hierarchy `revoke_root`
- lookup a root `get(root)`
- lookup a delegation `get(delegation)`
- lookup children of a delegation `get(children)`
on chain.

#### Create root
```rust
create_root(origin, root_id: T::DelegationNodeId, ctype_hash: T::Hash) -> Result
```
The `create_root` function takes the following parameters:

- origin: The caller of the method, i.e., public address (ss58) of the owner of the
 trust hierarchy
- root_id: A V4 UUID identifying the trust hierarchy
- ctype_hash: The blake2b hash of the CTYPE the trust hierarchy is associated with

The node verifies the transaction signature and insert it to the state. The root is stored by using
a map:

```rust
T::DelegationNodeId => (T::Hash,T::AccountId,bool)
```

#### Add delegation
```rust
add_delegation(origin, delegation_id: T::DelegationNodeId, root_id: T::DelegationNodeId, parent_id: Option<T::DelegationNodeId>, delegate: T::AccountId, permissions: Permissions, delegate_signature: T::Signature) -> Result
```

The `add_delegation` function takes the following parameters:
- origin: The caller of the method, i.e., public address (ss58) of the delegator
- delegation_id: A V4 UUID identifying this delegation
- root_id: A V4 UUID identifying the associated trust hierarchy
- parent_id: Optional, a V4 UUID identifying the parent delegation this delegation is
 based on
- CTYPEHash: The blake2b hash of CTYPE used when creating the Claim
- delegate: The public address (ss58) of the delegate (ID receiving the delegation)
- permissions: The permission bit set (having 0001 for attesting permission and
 0010 for delegation permission)
- delegate_signature: ed25519 based signature by the delegate based on the
 delegationId, rootId, parentId and permissions

The node verifies the transaction signature and the delegate signature as well as all other data
to be valid and the delegator to be permitted and then inserts it to the state. The delegation is
stored by using a map:
```rust
T::DelegationNodeId => (T::DelegationNodeId,Option<T::DelegationNodeId>,T::AccountId,Permissions,bool)
```

Additionally, if the delegation has a parent delegation, the information about the children of its
parent is updated in the following map that relates parents to their children:

```rust
T::DelegationNodeId => Vec<T::DelegationNodeId>
```

#### Revoke
```rust
revoke_root(origin, root_id: T::DelegationNodeId) -> Result
```
and
```rust
revoke_delegation(origin, delegation_id: T::DelegationNodeId) -> Result
```
