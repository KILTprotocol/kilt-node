# Decentralized Identity Provider (DIP) provider pallet

The Decentralized Identity Provider (DIP) provider pallet enables a Substrate-based chain (provider) to bridge the identities of its users to other connected chains (consumers) trustlessly.
A consumer chain is connected to a provider if there is a way for the consumer chain to verify proofs about parts of the state of the provider chain.

The pallet is agnostic over the chain-specific definition of identity and delegates the definition of it to the provider chain's runtime.

Because providers and consumers evolve at different speeds, identity commitments are versioned.
This allows the provider chain to upgrade to a newer commitment scheme while allowing users to use the old version if the chains on which they want to use their identity don't yet support the new scheme.

The identity subject can replace or remove identity commitments, for example, if something in the identity info changes.
After removal, the identity becomes unusable cross-chain, although it continues to exist on the provider chain and is usable for local operations.

## Add the pallet to the runtime

Add the pallet to runtime to the `Cargo.toml` file dependencies section:

```toml
provider = {package = "pallet-dip-provider", git = "https://github.com/KILTprotocol/kilt-node.git", branch = "release-1.12.0"}
```

The DIP pallet is available in the KILT node release 1.12.0 and later.

## Configure the pallet

Pass runtime configuration to the pallet's `Config` trait.

```ignore,rust
impl pallet_dip_provider::Config for Runtime {
    // Config
}
```

## The `Config` trait

As the runtime is chain-agnostic, the `Config` trait requires the following configuration parameters passed to it:

-   `type CommitOriginCheck: EnsureOrigin<Self::RuntimeOrigin, Success = Self::CommitOrigin>`: The check ensures a given runtime origin is allowed to generate and remove identity commitments.
-   `type CommitOrigin: SubmitterInfo<Submitter = Self::AccountId>`: The resulting origin if `CommitOriginCheck` returns without errors. The origin is not required to be an `AccountId`, but must include information about the `AccountId` of the tx submitter.
-   `type Identifier: Parameter + MaxEncodedLen`: The type of an identifier used to retrieve identity information about a subject.
-   `type IdentityCommitmentGenerator: IdentityCommitmentGenerator<Self>`: The type responsible for generating identity commitments, given the identity information associated with a given `Identifier`.
-   `type IdentityProvider: IdentityProvider<Self>`: The type responsible for retrieving the information associated with a subject given their identifier. The information can potentially be retrieved from any source, using a combination of on-chain and off-chain solutions.
-   `type IdentityProvider: IdentityProvider<Self>`: Customizable external logic to handle events in which a new identity commitment is generated or removed.
-   `type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>`: The aggregate `Event` type.

## Storage

The pallet stores identity commitments, which are opaque byte blobs in the pallet storage and on which runtime developers can build the cross-chain identity bridging protocol.

The runtime must define what an identity commitment looks like and is provider-specific.
Runtime developers must make this definition available to consumers willing to integrate the identities on the provider chain.

The pallet currently contains a single storage element, the `IdentityCommitments` double map.
Its first key is the `Identifier` of subjects, while the second key is the commitment version.
The values are identity commitments.
The double map allows the same subject to have one commitment for each version supported by the provider, without forcing consumers to upgrade to a new version to support the latest commitment scheme.

## Events

The pallet generates two events:

-   `VersionedIdentityCommited` called whenever a new commitment is stored, and contains information about the `Identifier` of the subject, the value of the commitment, and the commitment version.
-   `VersionedIdentityDeleted` called whenever a commitment is deleted, and contains information about the `Identifier` of the subject and the version of the commitment deleted.

## Calls

Bullet points represent each call's encoded index

0. `pub fn commit_identity(origin: OriginFor<T>, identifier: T::Identifier, version: Option<IdentityCommitmentVersion> ) -> DispatchResult`: Generate a new versioned commitment for the subject identified by the provided `Identifier`. If an old commitment for the same version is present, it is overridden. Hooks are called before the new commitment is stored, and optionally before the old one is replaced.
1. `pub fn delete_identity_commitment(origin: OriginFor<T>, identifier: T::Identifier, version: Option<IdentityCommitmentVersion>) -> DispatchResult`: Delete an identity commitment of a specific version for a specific `Identifier`. If a commitment of the provided version does not exist for the given Identifier, an error is returned. Hooks are called after the commitment has been removed.
