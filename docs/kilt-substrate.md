# Building on the Parity Substrate Blockchain Framework

During our first whiteboard phase, we were thinking about developing the KILT Protocol on
Ethereum smart-contracts, but we realized that we would have less **freedom of setting transaction costs**, while incurring a high level of overhead. 
Instead, we started our
development on [Parity Substrate](https://www.parity.io/substrate/), a general blockchain framework, and built up the KILT blockchain from scratch based on its module library and core functionality.

Building our blockchain on Parity Substrate has multiple advantages. Substrate has a very
good fundamental [architecture](https://substrate.dev/docs/en/knowledgebase/runtime/) and [codebase](https://github.com/paritytech/substrate) created by blockchain experts.
Moreover, the Substrate framework is developed in **Rust**, a memory efficient and fast compiled system programming
language, which provides a secure environment with virtually no runtime errors. 
Additionally, the node runtime is also compiled to **WebAssembly**, so older version native nodes can always run the latest version node runtime in a WebAssembly virtual machine to **bypass the problem of a blockchain fork**. 
Importantly, there is a vibrant developer community and rich [documentation](https://substrate.dev/).

Our implementation is based on the [substrate-node-template](https://github.com/substrate-developer-hub/substrate-node-template) library (skeleton template for quickly building a Substrate based Blockchain), which is linked to the main Substrate codebase.

## Helpful links

* [Polkadot Wiki Glossary](https://wiki.polkadot.network/docs/en/glossary)
* [Substrate Tutorials](https://substrate.dev/en/tutorials)
* [Substrate Documentation](https://substrate.dev/)
* [Substrate JSON-RPC API](https://polkadot.js.org/docs/substrate/rpc)
* [Substrate Reference Rust Docs](https://substrate.dev/rustdocs/v3.0.0/sc_service/index.html)
* [Substrate Recipes](https://substrate.dev/recipes/introduction.html)
* [Awesome Substrate - Collection of Substrate Tooling](https://substrate.dev/awesome-substrate/)
* [Substrate Playground](https://playground.substrate.dev/)

### Substrate Repositories

* [Substrate Repository](https://github.com/paritytech/substrate)
* [Polkadot Repository](https://github.com/paritytech/polkadot)
* [Cumulus Repository](https://github.com/paritytech/cumulus)

## Remote Procedure Calls

The Ethereum ecosystem highly leverages [JSON-RPC](https://www.jsonrpc.org/specification) where one can efficiently call methods and parameters directly on the blockchain node.
Based on good experiences, developers decided to use it in Substrate as well.
The [Polkadot API](https://polkadot.js.org/api/) helps with communicating with the JSON-RPC endpoint, and the clients and services never have to talk directly with the endpoint.

## Blocktime

The blocktime is currently set to [10 seconds](../runtimes/parachain/lib.rs#82), but this setting is subject to change based on further research.
We will consider what is affected by this parameter, and in the long term it will be fine-tuned to a setting that provides the best performance and user experience for the participants of the KILT network.

## Extrinsics and Block Storage

In Substrate, the blockchain transactions are abstracted away and are generalized as[extrinsics](https://docs.substrate.dev/docs/extrinsics) in the system.
They are called extrinsics since they can represent any piece of information that is regarded as input from “the outside world” (i.e. from users of the network) to the blockchain logic.
The blockchain transactions in KILT are implemented through these general extrinsics, that are signed by the originator of the transaction.
We use this framework to write the KILT Protocol specific data entries on the Substrate based KILT blockchain: DIDs, CTypes, Attestations and Delegations.
The processing of each of these entry types is handled by our custom Substrate modules called pallets.

Under the current consensus algorithm, authority validator nodes (whose addresses are listed in the genesis block) can create new blocks.
These nodes [validate](https://substrate.dev/docs/en/knowledgebase/learn-substrate/tx-pool#transaction-lifecycle) incoming transactions, put them into the pool, and include them in a new block.
While creating the block, the node executes the transactions and stores the resulting state changes in its local storage.
Note that the size of the entry depends on the number of arguments the transaction/respective extrinsic method has.
The size of the block is hence dynamic and will depend on the number and type of transactions included in the new block.
The valid new blocks are propagated through the network and other nodes execute these blocks to update their local state (storage).

## Authoring & Consensus Algorithm

We use [Aura](https://wiki.parity.io/Aura) as the authoring algorithm, since we are still in a permissioned blockchain mode.
We will probably move to another algorithm in the future (e.g. [BABE](https://w3f-research.readthedocs.io/en/latest/polkadot/BABE.html)).

For consensus we use [GRANDPA](https://github.com/paritytech/substrate#2-description).

At a later stage, we most likely will change to a different consensus algorithm that will incorporate additional features (e.g. proving availability of certain services) and we might leverage concepts from BABE+GRANDPA while designing this new consensus mechanism.

## Polkadot Integration

As a further great advantage, by basing ourselves on Substrate we can easily connect to the Polkadot ecosystem.
This could provide security for the KILT network by leveraging the global
consensus in the Polkadot network.
We are planning to integrate KILT into the [Polkadot](https://polkadot.network/) network.
It is fairly straightforward to achieve this by simply including specific Substrate modules into the KILT Collator node implementation.
The exact details of this integration is subject to future agreements between Polkadot and KILT and the technological development of Polkadot, Substrate and KILT.

## KILT Tokens

Coin transfers are implemented as a balance-based mechanism in Substrate. 
In our testnet, every new identity gets 1000 KILT Tokens from a root entity in the system who is wired into the genesis block.
At a later stage, we are proposing to provide KILT Tokens for new developers wanting to join our test networks (testnet and Spirit-Net testnet) on a simple request-provide based mechanism.
Preferably, developers will be able to register on our website, and we manually transfer KILT tokens to the registered developers after vetting them.
Importantly, these test tokens will not be usable on our mainnet.
After the launch of the mainnet (Spirit-Net) and the public KILT Token sale, the tokens will be available on cryptocurrency exchanges.