The KILT blockchain is the heart and soul behind KILT protocol. It provides the immutable
transaction ledger for the various processes in the network.

`Building on the Parity Substrate Blockchain Framework`

During our first whiteboard phase, we were thinking about developing the KILT protocol on
Ethereum smart-contracts, but we realised that we would have less freedom of setting
transaction costs, while incurring a high level of overhead. Instead, we started our
development on [Parity Substrate](https://www.parity.io/substrate/), a general blockchain framework, and built up the KILT blockchain from scratch based on its module library.

Building our blockchain on Parity Substrate has multiple advantages. Substrate has a very
good fundamental [architecture](https://substrate.dev/docs/en/knowledgebase/runtime/) and [codebase](https://github.com/paritytech/substrate) created by blockchain experts. Substrate
framework is developed in Rust, a memory efficient and fast compiled system programming
language, which provides a secure environment with virtually no runtime errors. Moreover, the
node runtime is also compiled to WebAssembly, so older version native nodes can always run
the latest version node runtime in a WebAssembly virtual machine to bypass the problem of a
blockchain fork. Importantly, there is a vibrant developer community and rich [documentation](https://substrate.dev/).

Our implementation is based on the [substrate-node-template](https://github.com/substrate-developer-hub/substrate-node-template) library (skeleton template for
quickly building a substrate based blockchain), which is linked to the main Substrate
codebase.

**Remote Procedure Calls**

The Ethereum ecosystem highly leverages [JSON-RPC](https://www.jsonrpc.org/specification) where one can efficiently call methods and parameters directly on the blockchain node. Based on good experiences, developers
decided to use it in Substrate as well. The [Polkadot API](https://polkadot.js.org/api/) helps with communicating with the JSON-RPC endpoint, and the clients and services never have to talk directly with the endpoint.

**Blocktime**

The blocktime is currently set to 5 seconds, but this setting is subject to change based on
further research. We will consider what is affected by this parameter, and in the long term it
will be fine-tuned to a setting that provides the best performance and user experience for the
participants of the KILT network.

**Extrinsics and Block Storage**

In Substrate, the blockchain transactions are abstracted away and are generalised as
[extrinsics](https://docs.substrate.dev/docs/extrinsics) in the system. They are called extrinsics since they can represent any piece of information that is regarded as input from “the outside world” (i.e. from users of the network) to the blockchain logic. The blockchain transactions in KILT are implemented through these
general extrinsics, that are signed by the originator of the transaction. We use this framework
to write the KILT Protocol specific data entries on the Substrate based KILT blockchain: [DID],
[CTYPEhash], [Attestation] and [Delegation]. The processing of each of these entry types is
handled by our custom Substrate runtime node modules.

Under the current consensus algorithm, authority validator nodes (whose addresses are listed
in the genesis block) can create new blocks. These nodes [validate](https://substrate.dev/docs/en/knowledgebase/learn-substrate/tx-pool#transaction-lifecycle) incoming transactions, put
them into the pool, and include them in a new block. While creating the block, the node
executes the transactions and stores the resulting state changes in its local storage. Note that
the size of the entry depends on the number of arguments the transaction, (i.e., the respective
extrinsic method) has. The size of the block is hence dynamic and will depend on the number
and type of transactions included in the new block. The valid new blocks are propagated
through the network and other nodes execute these blocks to update their local state (storage).

**Authoring & Consensus Algorithm**

We use [Aura](https://wiki.parity.io/Aura) as the authoring algorithm, since we are still in a permissioned blockchain mode.
We will probably move to another algorithm in the future (e.g. [BABE](https://w3f-research.readthedocs.io/en/latest/polkadot/BABE.html)).

For consensus we use [GRANDPA](https://github.com/paritytech/substrate#2-description).

At a later stage, we most likely will change to a different consensus algorithm that will incorporate additional features (e.g. proving availability of certain services) and we might leverage concepts from BABE+GRANDPA while designing this new consensus mechanism.

**Polkadot Integration**

As a further great advantage, by basing ourselves on Substrate we can easily connect to the
Polkadot ecosystem. This could provide security for the KILT network by leveraging the global
consensus in the Polkadot network. We are planning to integrate KILT into the [Polkadot](https://polkadot.network/)
network. It is fairly straightforward to achieve that by simply including specific Substrate
modules into the KILT Validator Node implementation. The exact details of this integration is
subject to future agreements between Polkadot and KILT and the technological development
of Polkadot, Substrate and KILT.

**KILT Tokens**

Coin transfer is implemented as a balance-based mechanism in Substrate. In our testnet,
every new identity gets 1000 KILT Tokens from a root entity in the system who is wired into
the genesis block. At a later stage in the testnet (Mash-Net) and the persistent testnet (WashNet), we are proposing to provide KILT Tokens for new developers wanting to join our network
on a simple request-provide based mechanism. Preferably, developers will be able to register
on our website, and we manually transfer KILT tokens to the registered developers.
Importantly, these test tokens will not be usable on our mainnet. After the launch of the mainnet
(Spirit-Net) and the public KILT Token sale, tokens will be available on cryptocurrency
exchanges.
