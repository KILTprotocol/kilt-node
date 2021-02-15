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
use sp_core::{sr25519, Pair};
use sp_std::{boxed::Box, vec, vec::Vec};


/// A default panic handler for WASM environment.
#[cfg(not(feature = "std"))]
#[panic_handler]
#[no_mangle]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
	unsafe {
		core::arch::wasm32::unreachable();
	}
}

/// A default OOM handler for WASM environment.
#[cfg(not(feature = "std"))]
#[alloc_error_handler]
pub fn oom(_: core::alloc::Layout) -> ! {
	unsafe {
		core::arch::wasm32::unreachable();
	}
}

const SEED: u32 = 0;

fn generate_delegation_id<T: Config>(_number: u64) -> T::DelegationNodeId {
	let delegation = <<T as Config>::DelegationNodeId as Default>::default();

	delegation
}

fn add_children<T: Config>(
	_root_acc: sr25519::Pair,
	_root_id: <T as Config>::DelegationNodeId,
	parent_acc: sr25519::Pair,
	parent_id: <T as Config>::DelegationNodeId,
	_ctype: <T as frame_system::Config>::Hash,
	levels: u64,
	children_per_level: u64,
) -> Result<(sr25519::Pair, T::DelegationNodeId), DispatchError>
where
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
{
	if levels == 0 {
		return Ok((parent_acc, parent_id));
	};

	let mut leaf = None;
	for c in 0..children_per_level {
		let node_acc = sr25519::Pair::from_seed_slice(&[0; 32])
			.map_err(|_| "Error while building node key pair.")?;
		let _node_acc_id: <T as frame_system::Config>::AccountId = node_acc.public().into();
		let node_id = generate_delegation_id::<T>(levels * children_per_level + c);

		// TODO: add delegation
		// Module::<T>::add_delegation(
		// 	RawOrigin::Signed(root_acc.clone()).into(),
		// 	delegation,
		// 	root,
		// 	ctype,
		// )

		// only put in a leaf in the the first iteration
		leaf = leaf.or(Some((node_acc, node_id)));
	}

	// TODO: only add childen for the first node

	// if we didn't add children, return the parent
	Ok(leaf.unwrap_or((parent_acc, parent_id)))
}

pub fn setup_delegations<T: Config>(
	levels: u64,
	children_per_level: u64,
) -> Result<
	(
		sr25519::Pair,
		T::DelegationNodeId,
		sr25519::Pair,
		T::DelegationNodeId,
	),
	DispatchError,
>
where
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
{
	// let root_acc: <T as frame_system::Config>::AccountId = account("root", 0, SEED);
	let root_acc = sr25519::Pair::from_seed_slice(&[0; 32])
		.map_err(|_| "Error while building root key pair.")?;
	let root_acc_id: <T as frame_system::Config>::AccountId = root_acc.public().into();

	let ctype = <<T as frame_system::Config>::Hash as Default>::default();
	ctype::Module::<T>::add(RawOrigin::Signed(root_acc_id.clone()).into(), ctype)?;

	let root_id = generate_delegation_id::<T>(0);
	Module::<T>::create_root(
		RawOrigin::Signed(root_acc_id.clone()).into(),
		root_id,
		ctype,
	)?;

	let (leaf_acc, leaf_id) = add_children::<T>(
		root_acc.clone(),
		root_id,
		root_acc.clone(),
		root_id,
		ctype,
		levels,
		children_per_level,
	)?;

	return Ok((root_acc, root_id, leaf_acc, leaf_id));
}

benchmarks! {
	where_clause { where T::Signature: From<sr25519::Signature>, <T as frame_system::Config>::AccountId: From<sr25519::Public> }

	create_root {
		let caller: <T as frame_system::Config>::AccountId = account("caller", 0, SEED);
		let ctype = <<T as frame_system::Config>::Hash as Default>::default();
		let delegation = generate_delegation_id::<T>(0);

		ctype::Module::<T>::add(RawOrigin::Signed(caller.clone()).into(), ctype)?;

	}: _(RawOrigin::Signed(caller), delegation, ctype)

	add_delegation {
		let (root_acc, root_id, leaf_acc, leaf_id) = setup_delegations::<T>(1, 2)?;
		let delegatee = sr25519::Pair::from_seed_slice(&[0; 32])
			.map_err(|_| "Error while building delegatee key pair.")?;
		let delegation_id = generate_delegation_id::<T>(u64::MAX);

		let perm: Permissions = Default::default();
		let hash_root = Module::<T>::calculate_hash(delegation_id, root_id, Some(leaf_id), perm);
		let sig: <T as Config>::Signature = delegatee.sign(hash_root.as_ref()).into();

		let delegatee_id: <T as frame_system::Config>::AccountId = delegatee.public().into();
		let leaf_acc_id: <T as frame_system::Config>::AccountId = leaf_acc.public().into();

	}: _(RawOrigin::Signed(leaf_acc_id), delegation_id, root_id, Some(leaf_id), delegatee_id, perm, sig)

	// revoke_root {
	// 	let caller = account("caller", 0, SEED);

	// }: _(RawOrigin::Signed(caller))

	// revoke_delegation {
	// 	let caller = account("caller", 0, SEED);

	// }: _(RawOrigin::Signed(caller))
}
