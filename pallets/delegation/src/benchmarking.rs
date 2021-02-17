// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019  BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_core::{offchain::KeyTypeId, sr25519};
use sp_io::crypto::sr25519_generate;
use sp_std::{boxed::Box, vec, vec::Vec};

const SEED: u32 = 0;

fn generate_delegation_id<T: Config>(number: u64) -> T::DelegationNodeId
where
	T::DelegationNodeId: From<<T as frame_system::Config>::Hash>,
{
	let hash: T::Hash = T::Hashing::hash(&number.to_ne_bytes());
	hash.into()
}

fn parent_id_check<T: Config>(
	root_id: T::DelegationNodeId,
	parent_id: T::DelegationNodeId,
) -> Option<T::DelegationNodeId> {
	if parent_id == root_id {
		None
	} else {
		Some(parent_id)
	}
}

fn add_children<T: Config>(
	root_id: <T as Config>::DelegationNodeId,
	parent_id: <T as Config>::DelegationNodeId,
	parent_acc_public: sr25519::Public,
	parent_acc_id: <T as frame_system::Config>::AccountId,
	level: u64,
	children_per_level: u64,
) -> Result<
	(
		sr25519::Public,
		<T as frame_system::Config>::AccountId,
		T::DelegationNodeId,
	),
	DispatchError,
>
where
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
	<T as Config>::Signature: From<sr25519::Signature>,
	T::DelegationNodeId: From<<T as frame_system::Config>::Hash>,
{
	if level == 0 {
		return Ok((parent_acc_public, parent_acc_id, parent_id));
	};

	let mut leaf = None;
	for c in 0..children_per_level {
		let delegation_acc_public = sr25519_generate(KeyTypeId(*b"aura"), None);
		let delegation_acc_id: <T as frame_system::Config>::AccountId =
			delegation_acc_public.into();
		let delegation_id = generate_delegation_id::<T>(level * children_per_level + c);

		frame_support::debug::RuntimeLogger::init();
		frame_support::debug::print!(
			"\niteration l{:?}/c{:?}\n root_id {:?},\n parent_id {:?},\nparent_acc_id {:?}\ndelegation_id {:?},\n delegation_acc_id {:?}",
			level,
			c,
			root_id,
			parent_id,
			parent_acc_id,
			delegation_id,
			delegation_acc_id
		);

		// only set parent if not root
		let parent = parent_id_check::<T>(root_id, parent_id);

		// sign
		let hash: Vec<u8> = Module::<T>::calculate_hash(
			delegation_id,
			root_id,
			parent,
			Permissions::ATTEST | Permissions::DELEGATE,
		)
		.encode();
		let sig: <T as Config>::Signature =
			sp_io::crypto::sr25519_sign(KeyTypeId(*b"aura"), &delegation_acc_public, hash.as_ref())
				.ok_or("Error while building signature of delegation.")?
				.into();

		// add delegation from delegate to parent
		let _ = Module::<T>::add_delegation(
			RawOrigin::Signed(parent_acc_id.clone()).into(),
			delegation_id,
			root_id,
			parent,
			delegation_acc_id.clone(),
			Permissions::ATTEST | Permissions::DELEGATE,
			sig,
		)?;

		// only put in a leaf in the the first iteration
		leaf = leaf.or(Some((
			delegation_acc_public,
			delegation_acc_id,
			delegation_id,
		)));
	}

	// TODO: only add childen for the first node

	// if we didn't add children, return the parent
	Ok(leaf.unwrap_or((parent_acc_public, parent_acc_id, parent_id)))
}

pub fn setup_delegations<T: Config>(
	levels: u64,
	children_per_level: u64,
) -> Result<
	(
		sr25519::Public,
		T::DelegationNodeId,
		sr25519::Public,
		T::DelegationNodeId,
	),
	DispatchError,
>
where
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
	<T as Config>::Signature: From<sr25519::Signature>,
	T::DelegationNodeId: From<<T as frame_system::Config>::Hash>,
{
	let root_public = sr25519_generate(KeyTypeId(*b"aura"), None);
	let root_acc: <T as frame_system::Config>::AccountId = root_public.into();

	let ctype = <<T as frame_system::Config>::Hash as Default>::default();
	ctype::Module::<T>::add(RawOrigin::Signed(root_acc.clone()).into(), ctype)?;

	let root_id = generate_delegation_id::<T>(0);
	Module::<T>::create_root(
		RawOrigin::Signed(root_public.clone().into()).into(),
		root_id,
		ctype,
	)?;

	// iterate levels and start with root
	let mut leaf_acc_public = root_public;
	let mut leaf_acc_id = root_acc;
	let mut leaf_id = root_id;
	for l in 0..=levels {
		let (leaf_acc_public, leaf_acc_id, leaf_id) = add_children::<T>(
			root_id,
			leaf_id,
			leaf_acc_public,
			leaf_acc_id.clone(),
			l,
			children_per_level,
		)?;
	}

	return Ok((root_public, root_id, leaf_acc_public, leaf_id));
}

benchmarks! {
	where_clause { where T: core::fmt::Debug, T::Signature: From<sr25519::Signature>, <T as frame_system::Config>::AccountId: From<sr25519::Public>, 	T::DelegationNodeId: From<<T as frame_system::Config>::Hash> }

	create_root {
		let caller: <T as frame_system::Config>::AccountId = account("caller", 0, SEED);
		let ctype = <<T as frame_system::Config>::Hash as Default>::default();
		let delegation = generate_delegation_id::<T>(0);

		ctype::Module::<T>::add(RawOrigin::Signed(caller.clone()).into(), ctype)?;

	}: _(RawOrigin::Signed(caller), delegation, ctype)

	add_delegation {
		let (_, root_id, leaf_acc, leaf_id) = setup_delegations::<T>(1, 1)?;
		let delegate_acc_public = sr25519_generate(
			KeyTypeId(*b"aura"),
			None
		);
		frame_support::debug::print!("Done with setup! \n Root_id {:?} \n Leaf_id{:?}", root_id, leaf_id);
		let delegation_id = generate_delegation_id::<T>(u64::MAX);
		let parent_id = parent_id_check::<T>(root_id, leaf_id);

		let perm: Permissions = Permissions::ATTEST | Permissions::DELEGATE;
		let hash_root = Module::<T>::calculate_hash(delegation_id, root_id, parent_id, perm);
		let sig: <T as Config>::Signature = sp_io::crypto::sr25519_sign(KeyTypeId(*b"aura"), &delegate_acc_public, hash_root.as_ref()).ok_or("Error while building signature of delegation.")?.into();

		let delegate_acc_id: <T as frame_system::Config>::AccountId = delegate_acc_public.into();
		let leaf_acc_id: <T as frame_system::Config>::AccountId = leaf_acc.into();

	}: _(RawOrigin::Signed(leaf_acc_id), delegation_id, root_id, parent_id, delegate_acc_id, perm, sig)

	// revoke_root {
	// 	let caller = account("caller", 0, SEED);

	// }: _(RawOrigin::Signed(caller))

	// revoke_delegation {
	// 	let caller = account("caller", 0, SEED);

	// }: _(RawOrigin::Signed(caller))
}
