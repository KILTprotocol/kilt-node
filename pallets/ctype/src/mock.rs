// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use frame_support::{parameter_types, weights::constants::RocksDbWeight};
use sp_core::H256;
use sp_keystore::{testing::KeyStore, KeystoreExt};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};
use sp_std::sync::Arc;

use crate as ctype;
use crate::*;

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

pub type TestCtypeOwner = kilt_primitives::AccountId;
pub type TestCtypeHash = kilt_primitives::Hash;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Ctype: ctype::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = kilt_primitives::Hash;
	type Hashing = BlakeTwo256;
	type AccountId = <<kilt_primitives::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = ();

	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

impl Config for Test {
	type FeeHandler = ();
	type CtypeCreatorId = TestCtypeOwner;
	type EnsureOrigin = frame_system::EnsureSigned<TestCtypeOwner>;
	type OriginSuccess = TestCtypeOwner;
	type Event = ();
	type WeightInfo = ();
}

#[cfg(test)]
pub(crate) const ALICE: TestCtypeOwner = TestCtypeOwner::new([0u8; 32]);

const DEFAULT_CTYPE_HASH_SEED: u64 = 1u64;
const ALTERNATIVE_CTYPE_HASH_SEED: u64 = 2u64;

pub fn get_origin(account: TestCtypeOwner) -> Origin {
	Origin::signed(account)
}

pub fn get_ctype_hash<T>(default: bool) -> CtypeHashOf<T>
where
	T: Config,
	T::Hash: From<H256>,
{
	if default {
		H256::from_low_u64_be(DEFAULT_CTYPE_HASH_SEED).into()
	} else {
		H256::from_low_u64_be(ALTERNATIVE_CTYPE_HASH_SEED).into()
	}
}

#[derive(Clone, Default)]
pub struct ExtBuilder {
	ctypes_stored: Vec<(TestCtypeHash, TestCtypeOwner)>,
}

impl ExtBuilder {
	pub fn with_ctypes(mut self, ctypes: Vec<(TestCtypeHash, TestCtypeOwner)>) -> Self {
		self.ctypes_stored = ctypes;
		self
	}

	pub fn build(self, ext: Option<sp_io::TestExternalities>) -> sp_io::TestExternalities {
		let mut ext = if let Some(ext) = ext {
			ext
		} else {
			let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			sp_io::TestExternalities::new(storage)
		};

		if !self.ctypes_stored.is_empty() {
			ext.execute_with(|| {
				self.ctypes_stored.iter().for_each(|ctype| {
					ctype::Ctypes::<Test>::insert(ctype.0, ctype.1.clone());
				})
			});
		}

		ext
	}

	pub fn build_with_keystore(self, ext: Option<sp_io::TestExternalities>) -> sp_io::TestExternalities {
		let mut ext = self.build(ext);

		let keystore = KeyStore::new();
		ext.register_extension(KeystoreExt(Arc::new(keystore)));

		ext
	}
}
