### DID Module

The KILT blockchain node runtime defines an DID module exposing:

#### Add

```rust
add(origin, sign_key: T::PublicSigningKey, box_key: T::PublicBoxKey, doc_ref: Option<Vec<u8>>) -> Result
```

This function takes the following parameters:

- origin: public [ss58](<https://wiki.parity.io/External-Address-Format-(SS58)>) address of the caller of the method
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

- origin: public [ss58](https://substrate.dev/docs/en/knowledgebase/advanced/ss58-address-format) address of the caller of the method
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

- origin: The caller of the method, i.e., public address ([ss58](https://substrate.dev/docs/en/knowledgebase/advanced/ss58-address-format)) of the Attester
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