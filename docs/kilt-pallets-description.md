# Description of KILT Pallets

Currently, the KILT runtime uses the following four custom pallets for handling the underlying KILT data structures: 
* [`did`](../pallets/did)
* [`attestation`](../pallets/attestation)
* [`delegation`](../pallets/delegation)
* [`ctype`](../pallets/ctype)

## The KILT DID Pallet

The KILT blockchain node runtime defines a DID module exposing:

### Add a DID

```rust
fn add(origin, sign_key: T::PublicSigningKey, box_key: T::PublicBoxKey, doc_ref: Option<Vec<u8>>) -> DispatchResult
```

This transaction takes the following parameters:

- `origin`: The public [ss58](<https://substrate.dev/docs/en/knowledgebase/advanced/ss58-address-format>) address of the caller of the method.
- `sign_key`: The [ed25519](http://ed25519.cr.yp.to/) or [sr25519](https://wiki.polkadot.network/docs/en/learn-cryptography) public signing key of the owner.
- `box_key`: The [x25519-xsalsa20-poly1305](http://nacl.cr.yp.to/valid.html) public encryption key of the owner.
- `doc_ref`: An optional u8 byte vector representing the reference (URL) to the DID document.

The blockchain node verifies the transaction signature corresponding to the owner and inserts it to the blockchain storage by using a map (done by the substrate framework):

```rust
struct DidRecord<T> {
  sign_key: T::PublicSigningKey,
  box_key: T::PublicBoxKey,
  doc_ref: Option<Vec<u8>>
}

type DidStorage<T> = StorageMap<T::AccountId, Option<DidRecord<T>>>
```

### Remove a DID

```rust
fn remove(origin) -> DispatchResult
```

This function takes the owner as a single parameter removes the DID from the storage map.
Thus, any any later read operation call does not return the data of a removed DID.

## The KILT CType Pallet

The KILT blockchain node runtime defines a CType module exposing

```rust
fn add(origin, hash: T::Hash) -> DispatchResult
```

This function takes following parameters:

- `origin`: The public [ss58](https://substrate.dev/docs/en/knowledgebase/advanced/ss58-address-format) address of the caller of the method
- `hash`: The CType hash as a [blake2b](https://blake2.net/) string

The blockchain node verifies the transaction signature corresponding to the creator and
inserts it to the blockchain storage by using a map (done by the substrate framework):

```rust
type CTypeStorage<T> = StorageMap<T::Hash, Option<T::AccountId>>
```

## The KILT Attestation Pallet

The KILT blockchain node runtime defines an Attestation module exposing functions to 

- add a (delegated) attestation `add`
- revoke a (delegated) attestation `revoke`
- lookup an attestation `get(attestation)`
- lookup a delegated attestation `get(delegated_attestations)`

on chain.

### Add an Attestation

```rust
fn add(origin, claim_hash: T::Hash, ctype_hash: T::Hash, delegation_id: Option<T::DelegationNodeId>) -> DispatchResult
```

The `add` function takes following parameters:

- `origin`: The caller of the method, i.e. the public address ([ss58](https://substrate.dev/docs/en/knowledgebase/advanced/ss58-address-format)) of the (delegated) Attester.
- `claim_hash`: The Claim hash as [blake2b](https://blake2.net/) string used as the key of the entry.
- `ctype_hash`: The [blake2b](https://blake2.net/) hash of CType used when creating the Claim.
- `delegation_id`: An optional reference to a Delegation which this attestation is based on.

The node verifies the transaction signature and inserts it to the state, if the provided attester
didn’t already attest the provided claimHash.
The attestation is stored by using a map:

```rust
struct Attestation<T> {
	ctype_hash: T::Hash,
	attester: T::AccountId,
	delegation_id: Option<T::DelegationNodeId>,
	revoked: bool,
}

type AttestationStorage<T> = StorageMap<T::Hash, Option<Attestation<T>>>
```

Delegated Attestations are stored in an additional map:

```rust
type DelegatedAttestationStorage<T> = StorageMap<T::DelegationNodeId, Vec<T::Hash>>
```

### Revoke an Attestation

```rust
fn revoke(origin, claim_hash: T::Hash, max_depth: u32) -> DispatchResult
```

- `origin`: The caller of the method, i.e. the public address ([ss58](https://substrate.dev/docs/en/knowledgebase/advanced/ss58-address-format)) of the (delegated) Attester.
- `claim_hash`: The Claim hash as [blake2b](https://blake2.net/) string used as the key of the entry.
- `max_depth`: The maximum number of parent checks of the Delegation which are supported in this call until finding the owner of the Delegation (or the Root) and ensuring that this is the address of the `origin`. Due to the recursive structure of checking for Delegations, this kind of limit is required for calculating the transaction fees and the weight of the transaction.

The `revoke` function takes the claim hash (which is the key to lookup an attestation) as argument.
After looking up the attestation and checking the invoker's permissions, the revoked flag is set to true and the updated attestation is stored on chain.

## The KILT Delegation Pallet

The KILT blockchain node runtime defines a Delegation module exposing functions to

- create a root delegation `create_root`
- add a delegation `add_delegation`
- revoke a delegation `revoke_delegation`
- revoke a whole hierarchy `revoke_root`
- lookup a root `get(root)`
- lookup a delegation `get(delegation)`
- lookup children of a delegation `get(children)` 

on chain.

### Create a Root Delegation

```rust
fn create_root(origin, root_id: T::DelegationNodeId, ctype_hash: T::Hash) -> DispatchResult
```

The `create_root` function takes the following parameters:

- `origin`: The caller of the method, i.e., public address (ss58) of the owner of the trust hierarchy.
- `root_id`: A V4 UUID identifying the trust hierarchy.
- `ctype_hash`: The [blake2b](https://blake2.net/) hash of the CType this trust hierarchy is associated with. 

The node verifies the transaction signature and inserts it to the state.
The root is stored by using a map:

```rust
struct DelegationRoot<T: Config> {
	ctype_hash: T::Hash,
	owner: T::AccountId,
	revoked: bool,
}
type RootStorage = StorageMap<T::DelegationNodeId, Option<DelegationRoot<T>>>
```

### Add a Delegation

```rust
fn add_delegation(
	origin,
	delegation_id: T::DelegationNodeId,
	root_id: T::DelegationNodeId,
	parent_id: Option<T::DelegationNodeId>,
	delegate: T::AccountId,
	permissions: Permissions,
	delegate_signature: T::Signature
) -> DispatchResult
```

The `add_delegation` function takes the following parameters:

- `origin`: The caller of the method, i.e. the public address (ss58) of the Delegator.
- `delegation_id`: A V4 UUID identifying this delegation.
- `root_id`: A V4 UUID identifying the associated trust hierarchy.
- `parent_id`: An Optional V4 UUID identifying the parent delegation this delegation is based on.
- `delegate`: The public address (ss58) of the Delegate (ID receiving the delegation).
- `permissions`: The permission bit set (having `0001` for attesting permission and `0010` for delegation permission)
- `delegate_signature`: An `ed25519` or `sr25519` based signature by the delegate based on the `delegationId`, `rootId`, `parentId` and `permissions`.

The node verifies the transaction signature and the delegate signature as well as all other data
to be valid and the delegator to be permitted and then inserts it to the state. The delegation is
stored by using a map:

```rust
struct DelegationNode<T: Config> {
	root_id: T::DelegationNodeId,
	parent: Option<T::DelegationNodeId>,
	owner: T::AccountId,
	permissions: Permissions,
	revoked: bool,
}
type DelegationsStorage = StorageMap<T::DelegationNodeId, Option<DelegationNode<T>>>
```

Additionally, if the delegation has a parent delegation, the information about the children of its
parent is updated in the following map that relates parents to their children:

```rust
type ChildrenStorage = StorageMap<T::DelegationNodeId, Vec<T::DelegationNodeId>>
```

### Revoke a DelegationRoot

```rust
fn revoke_root(origin, root_id: T::DelegationNodeId, max_children: u32) -> DispatchResultWithPostInfo {
```

The `revoke_root` function takes the following parameters:

- `origin`: The caller of the method, i.e. the public address (ss58) of the Delegator.
- `root_id`: A V4 UUID identifying the associated trust hierarchy.
- `max_children`: The maximum number of children of the Delegation root which can be revoked with this call. Due to the recursive structure of checking for Children, this kind of limit is required for calculating the transaction fees and the weight of the transaction.

### Revoke a Delegation

```rust
fn revoke_delegation(origin, delegation_id: T::DelegationNodeId, max_depth: u32, max_revocations: u32) -> DispatchResultWithPostInfo {
```

The `revoke_delegation` function takes the following parameters:

- `origin`: The caller of the method, i.e. the public address (ss58) of the Delegator.
- `delegation_id`: A V4 UUID identifying this delegation.
- `max_depth`: The maximum number of parent checks of the Delegation which are supported in this call until finding the owner of the Delegation (or the Root) and ensuring that this is the address of the `origin`. Due to the recursive structure of checking for Delegations, this kind of limit is required (at least) for benchmarks.
- `max_revocations`: The maximum number of children of this Delegation node which can be revoked with this call. Due to the recursive structure of checking for Children, this kind of limit is required for calculating the transaction fees and the weight of the transaction.