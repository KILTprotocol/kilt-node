# KILT Decentralized Identity Provider (DIP) provider specification

Specification of the format of a DIP identity commitment and the expected format of a DIP identity proof for cross-chain transactions using KILT identities.

## V0

The V0 of the KILT DIP Provider specification defines the following components:

* **Identity details**: What are the pieces of a KILT identity that can be used for cross-chain transactions. V0 defines them to include the following information:
  * All `DidKey`s stored under the subject's DID Document. For more details about how these keys are defined, read the [KILT DID pallet](../../../../pallets/did).
  * All the `LinkableAccountId`s the DID subject has linked to the DID via the KILT linking pallet. For more details about how on-chain linking works, read the [KILT lookup pallet](../../../../pallets/pallet-did-lookup/).
  * (OPTIONAL) The web3name of the DID subject, if present. For more details about how web3names work, read the [KILT web3name pallet](../../../../pallets/pallet-web3-names/).
* **Identity commitment**: Defines how the identity details above are aggregated into a value which will be selectively shared on a consumer chain for a cross-chain transaction. V0 defines the identity commitment as a Merkle root of all the elements above that uses the shame hashing algorithm as the runtime. Using a Merkle root allows the DID subject to generate proof that can selectively disclose different pieces of identity for different operations on different chains providing, among other things, better scalability for cases in which the linked information becomes large. The leaves encoded in the commitment can be of the following type:
  * DID key leaf: with leaf name being the key ID, and leaf value being the key details as defined in the `DidPublicKeyDetails` type.
  * Linked account leaf: with leaf name being the linked account ID, and leaf value being an empty tuple `()`.
  * Web3name leaf: with leaf name being the web3name, and leaf value being the KILT block number in which it was linked to the DID.
