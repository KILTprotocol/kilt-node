// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

// From Polkadot open PR https://github.com/paritytech/polkadot/pull/6662

use codec::Encode;
use sp_io::hashing::blake2_256;
use sp_std::{borrow::Borrow, marker::PhantomData};
use xcm::latest::prelude::*;
use xcm_executor::traits::Convert;

/// Prefix for generating alias account for accounts coming
/// from chains that use 32 byte long representations.
pub const FOREIGN_CHAIN_PREFIX_PARA_32: [u8; 37] = *b"ForeignChainAliasAccountPrefix_Para32";

/// Prefix for generating alias account for accounts coming
/// from chains that use 20 byte long representations.
pub const FOREIGN_CHAIN_PREFIX_PARA_20: [u8; 37] = *b"ForeignChainAliasAccountPrefix_Para20";

/// Prefix for generating alias account for accounts coming
/// from the relay chain using 32 byte long representations.
pub const FOREIGN_CHAIN_PREFIX_RELAY: [u8; 36] = *b"ForeignChainAliasAccountPrefix_Relay";

/// This converter will for a given `AccountId32`/`AccountKey20`
/// always generate the same "remote" account for a specific
/// sending chain.
/// I.e. the user gets the same remote account
/// on every consuming para-chain and relay chain.
///
/// Can be used as a converter in `SovereignSignedViaLocation`
pub struct ForeignChainAliasAccount<AccountId>(PhantomData<AccountId>);
impl<AccountId: From<[u8; 32]> + Clone> Convert<MultiLocation, AccountId> for ForeignChainAliasAccount<AccountId> {
	fn convert_ref(location: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		let entropy = match location.borrow() {
			// Used on the relay chain for sending paras that use 32 byte accounts
			MultiLocation {
				parents: 0,
				interior: X2(Parachain(para_id), AccountId32 { id, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_para_32(para_id, id),

			// Used on the relay chain for sending paras that use 20 byte accounts
			MultiLocation {
				parents: 0,
				interior: X2(Parachain(para_id), AccountKey20 { key, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_para_20(para_id, key),

			// Used on para-chain for sending paras that use 32 byte accounts
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(para_id), AccountId32 { id, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_para_32(para_id, id),

			// Used on para-chain for sending paras that use 20 byte accounts
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(para_id), AccountKey20 { key, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_para_20(para_id, key),

			// Used on para-chain for sending from the relay chain
			MultiLocation {
				parents: 1,
				interior: X1(AccountId32 { id, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_relay_32(id),

			// No other conversions provided
			_ => return Err(()),
		};

		Ok(entropy.into())
	}

	fn reverse_ref(_: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		Err(())
	}
}

impl<AccountId> ForeignChainAliasAccount<AccountId> {
	fn from_para_32(para_id: &u32, id: &[u8; 32]) -> [u8; 32] {
		(FOREIGN_CHAIN_PREFIX_PARA_32, para_id, id).using_encoded(blake2_256)
	}

	fn from_para_20(para_id: &u32, id: &[u8; 20]) -> [u8; 32] {
		(FOREIGN_CHAIN_PREFIX_PARA_20, para_id, id).using_encoded(blake2_256)
	}

	fn from_relay_32(id: &[u8; 32]) -> [u8; 32] {
		(FOREIGN_CHAIN_PREFIX_RELAY, id).using_encoded(blake2_256)
	}
}
