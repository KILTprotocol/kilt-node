# Decentralized Identity Provider (DIP) provider consumer pallet

This pallet is a core component of the Decentralized Identity Provider protocol.
It enables entities with an identity on a connected Substrate-based chain (provider) to use those identities on the chain this pallet is deployed (consumers) without requiring those entities to set up a new identity locally.
A consumer chain is *connected* to a provider if there is a way for the consumer chain to verify state proofs about parts of the state of the provider chain.

A cross-chain transaction with DIP assumes the entity submitting the transaction has already generated a cross-chain identity commitment on the provider chain, by interacting with the DIP provider pallet on the provider chain.
With a generated identity commitment, a cross-chain transaction flow for a generic entity `A` works as follows:

1. `A` generates a state proof proving the state of the identity commitment on the provider chain.
2. `A` generates any additional information required for an identity proof to be successfully verified by the consumer runtime.
3. `A`, using their account `AccC` on the consumer chain, calls the `dispatch_as` extrinsic by providing its identifier on the provider chain, the generated proof, and the `Call` to be dispatched on the consumer chain.
   1. This pallet verifies if the proof is correct, if not it returns an error.
   2. This pallet dispatches the provided `Call` with a new origin created by this pallet, returning any errors the dispatch action returns. The origin contains the information revealed in the proof, the identifier of the acting subject and the account `AccC` dispatching the transaction.

The pallet is agnostic over the chain-specific definition of *identity proof verifier* and *identifier*, although, when deployed, they must be configured to respect the definition of identity and identity commitment established by the provider this pallet is linked to.

For instance, if the provider establishes that an identity commitment is a Merkle root of a set of public keys, an identity proof for the consumer will most likely be a Merkle proof revealing a subset of those keys.
Similarly, if the provider defines an identity commitment as some ZK-commitment, the respective identity proof on the consumer chain will be a ZK-proof verifying the validity of the commitment and therefore of the revealed information.

For identifiers, if the provider establishes that an identifier is a public key, the same definition must be used in the consumer pallet.
Other definitions for an identifier, such as a simple integer or a [Decentralized Identifier (DID)](https://www.w3.org/TR/did-core/), must also be configured in the same way.

The pallet allows the consumer runtime to define some `LocalIdentityInfo` associated with each identifier, which the pallet's proof verifier can access and optionally modify upon proof verification.
Any changes made to the `LocalIdentityInfo` will be persisted if the identity proof is verified correctly and the extrinsic executed successfully.

If the consumer does not need to store anything in addition to the information an identity proof conveys, they can use an empty tuple `()` for the local identity info.
Another example could be the use of signatures, which requires a nonce to avoid replay protections.
In this case, a numeric type such as a `u64` or a `u128` could be used, and increased by the proof verifier when validating each new cross-chain transaction proof.

## The `Config` trait

Being chain-agnostic, most of the runtime configurations must be passed to the pallet's `Config` trait.
Nevertheless, most of the types provided must reflect the definition of identity and identity commitment that the identity provider chain has established.
The trait has the following components:

* `type DipCallOriginFilter: Contains<RuntimeCallOf<Self>>`: A preliminary filter that checks whether a provided `Call` accepts a DIP origin or not. If a call such as a system call does not accept a DIP origin, there is no need to verify the identity proof, hence the execution can bail out early. This does not guarantee that the dispatch call will succeed, but rather than it will mostly not fail with a `BadOrigin` error.
* `type DispatchOriginCheck: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin, Success = Self::AccountId>`: The origin check on the `dispatch_as` extrinsic to verify that the caller is authorized to call the extrinsic. If successful, the check must return a `AccountId` as defined by the consumer runtime.
* `type Identifier: Parameter + MaxEncodedLen`: The type of a subject identifier. This must match the definition of `Identifier` the identity provider has defined in their deployment of the provider pallet.
* `type LocalIdentityInfo: FullCodec + TypeInfo + MaxEncodedLen`: Any additional information that must be available only to the provider runtime that is required to provide additional context when verifying a cross-chain identity proof.
* `type ProofVerifier: IdentityProofVerifier<Self>`: The core component of this pallet. It takes care of validating an identity proof and optionally update any `LocalIdentityInfo`. It also defines, via its associated type, the structure of the identity proof that must be passed to the `dispatch_as` extrinsic. Although not directly, the proof structure depends on the information that goes into the identity commitment on the provider chain, as that defines what information can be revealed as part of the commitment proof. Additional info to satisfy requirements according to the `LocalIdentityInfo` (e.g., a signature) must also be provided in the proof.
* `type RuntimeCall: Parameter + Dispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>`: The aggregated `Call` type.
* `type RuntimeOrigin: From<Origin<Self>> + From<<Self as frame_system::Config>::RuntimeOrigin>`: The aggregated `Origin` type, which must include the origin exposed by this pallet.

## Storage

The pallet contains a single storage element, the `IdentityEntries` map.
It maps from a subject `Identifier` to an instance of `LocalIdentityInfo`.

This information is updated by the proof verifier whenever a new cross-chain transaction and its proof is submitted.

## Origin

Because the pallet allows other `Call`s to be dispatched after an identity proof has been verified, it also exposes a `Origin` that can be used for those calls that require indeed a call to be DIP-authorized.

The origin is created after the identity proof has been successfully verified by the proof verifier, and it includes the identifier of the subject, the address of the tx submitter, and the result returned by the proof verifier upon successful verification.

## Calls (bullet numbers represent each call's encoded index)

0. `pub fn dispatch_as(origin: OriginFor<T>, identifier: T::Identifier, proof: IdentityProofOf<T>, call: Box<RuntimeCallOf<T>>) -> DispatchResult`: Try to dispatch a new local call only if it passes all the DIP requirements. Specifically, the call will be dispatched if it passes the preliminary `DipCallOriginFilter` and if the proof verifier returns an `Ok(verification_result)` value. The value is then added to the `DipOrigin` and passed down as the origin for the specified `Call`. If the whole execution terminates successfully, any changes applied to the `LocalIdentityInfo` by the proof verifier are persisted to the pallet storage.
