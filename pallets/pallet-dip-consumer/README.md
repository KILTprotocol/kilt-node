# Decentralized Identity Provider (DIP) provider consumer pallet

This pallet is a component of the DIP protocol.
It enables entities with an identity on a connected Substrate-based chain (provider) to use those identities on the chain this pallet is deployed (consumers) without requiring those entities to set up a new identity locally.

A consumer chain is _connected_ to a provider if there is a way for the consumer chain to verify proofs about parts of the state of the provider chain.

A cross-chain transaction with DIP assumes the entity submitting the transaction has already generated a cross-chain identity commitment on the provider chain, by interacting with the DIP provider pallet on the provider chain.
With a generated identity commitment, a cross-chain transaction flow for a generic entity `A` works as follows:

1. `A` generates a state proof proving the state of the identity commitment on the provider chain.
2. `A` generates any additional information required for the consumer runtime to successfully verify an identity proof.
3. `A`, using their account `AccC` on the consumer chain `C`, calls the `dispatch_as` extrinsic by providing its identifier on the provider chain, the generated proof, and the `Call` to be dispatched on the consumer chain.

    1. This pallet verifies if the proof is correct, if not it returns an error.
    2. This pallet dispatches the provided `Call` with a new origin created by this pallet, returning any errors the dispatch action returns. The origin contains the information revealed in the proof, the identifier of the acting subject and the account `AccC` dispatching the transaction.

The pallet is agnostic over the chain-specific definition of _identity proof verifier_ and _identifier_, although, when deployed, runtime developers must configure them to respect the definition of identity and identity commitment established by the provider linked to this pallet.

For instance, if the provider establishes that an identity commitment is a Merkle root of a set of public keys, an identity proof for the consumer is most likely a Merkle proof revealing a subset of those keys.
Similarly, if the provider defines an identity commitment as some ZK commitment, the respective identity proof on the consumer chain is a ZK proof verifying the validity of the commitment and the revealed information.

For identifiers, if the provider establishes that an identifier is a public key, the consumer pallet must use the same definition.
Runtime developers must configure other definitions for an identifier, such as a simple integer or a [Decentralized Identifier (DID)](https://www.w3.org/TR/did-core/) in the same way.

The pallet allows the consumer runtime to define some `LocalIdentityInfo` associated with each identifier, which is additional information the pallet's proof verifier can access and optionally change upon proof verification.
Any changes made to the `LocalIdentityInfo` is persisted if the identity proof is verified correctly and the extrinsic ran successfully.

If the consumer does not need to store anything in addition to the information an identity proof conveys, they can use an empty tuple `()` for the local identity info.
Another example could be the use of signatures, which requires a nonce to avoid replay protections.
In this case, use a numeric type such as a `u64` or a `u128` and the proof verifier increases it when validating each new cross-chain transaction proof.

## Add the pallet to the runtime

Add the pallet to runtime to the `Cargo.toml` file dependencies section:

```toml
consumer = {package = "pallet-dip-consumer", git = "https://github.com/KILTprotocol/kilt-node.git", branch = "release-1.12.0"}
```

The DIP pallet is available in the KILT node release 1.12.0 and later.

## The `Config` trait

Pass runtime configuration to the pallet's `Config` trait.

```rust,ignore
impl pallet_dip_provider::Config for Runtime {
    // Config
}
```

As the runtime is chain-agnostic, the `Config` trait requires the following configuration parameters passed to it. Most of the types provided must reflect the definition of identity and identity commitment that the identity provider chain has established.

The trait has the following components:

-   `type DipCallOriginFilter: Contains<RuntimeCallOf<Self>>`: A preliminary filter that checks whether a provided `Call` accepts a DIP origin or not. If a call such as a system call does not accept a DIP origin, there is no need to verify the identity proof, hence the execution can bail out early. This does not guarantee that the dispatch call succeeds, but likely not fail with a `BadOrigin` error.
-   `type DispatchOriginCheck: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin, Success = Self::AccountId>`: The origin check on the `dispatch_as` extrinsic to verify that the caller is authorized to call the extrinsic. If successful, the check must return an `AccountId` as defined by the consumer runtime.
-   `type Identifier: Parameter + MaxEncodedLen`: The subject identifier type. This must match the definition of `Identifier` the identity provider has defined in their deployment of the provider pallet.
-   `type LocalIdentityInfo: FullCodec + TypeInfo + MaxEncodedLen`: Any additional information that must be available only to the provider runtime required to provide context when verifying a cross-chain identity proof.
-   `type ProofVerifier: IdentityProofVerifier<Self>`: The core component of this pallet that takes care of validating an identity proof and optionally updates any `LocalIdentityInfo`. It also defines, via its associated type, the structure of the identity proof that must be passed to the `dispatch_as` extrinsic. Although not directly, the proof structure depends on the information that goes into the identity commitment on the provider chain, as that defines what information can be revealed as part of the commitment proof. Additional info to satisfy requirements according to the `LocalIdentityInfo` (e.g., a signature) must also be provided in the proof.
-   `type RuntimeCall: Parameter + Dispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>`: The aggregated `Call` type.
-   `type RuntimeOrigin: From<Origin<Self>> + From<<Self as frame_system::Config>::RuntimeOrigin>`: The aggregated `Origin` type, which must include the origin exposed by this pallet.

## Storage

The pallet contains a single storage element, the `IdentityEntries` map. It maps from a subject `Identifier` to an instance of `LocalIdentityInfo`.

The proof verifier updates this information whenever a new cross-chain transaction and its proof is submitted.

## Origin

Because the pallet allows the dispatching of other `Call`s after an identity proof has been verified, it also exposes an `Origin` to calls that need to be DIP-authorized.

After the proof verifier has successfully verified the identity proof, the origin is created, and it includes the identifier of the subject, the address of the tx submitter, and the result returned by the proof verifier upon successful verification.

## Calls

Bullet points represent each call's encoded index

0. `pub fn dispatch_as(origin: OriginFor<T>, identifier: T::Identifier, proof: IdentityProofOf<T>, call: Box<RuntimeCallOf<T>>) -> DispatchResult`: Try to dispatch a new local call only if it passes all the DIP requirements. Specifically, the call is dispatched if it passes the preliminary `DipCallOriginFilter` and if the proof verifier returns an `Ok(verification_result)` value. The value is then added to the `DipOrigin` and passed down as the origin for the specified `Call`. If the whole execution terminates successfully, any changes applied to the `LocalIdentityInfo` by the proof verifier are persisted to the pallet storage.
