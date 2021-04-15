use crate as ctype;
use crate::*;
use did::mock as did_mock;

use frame_support::{parameter_types, weights::constants::RocksDbWeight};
use kilt_primitives::{AccountId, Signature};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Ctype: ctype::{Pallet, Call, Storage, Event<T>},
		Did: did::{Pallet, Call, Storage, Event<T>},
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
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
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
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

impl Config for Test {
	type Event = ();
	type WeightInfo = ();
}

impl did::Config for Test {
	type Event = ();
	type WeightInfo = ();
	type DidIdentifier = AccountId;
}

pub type TestCtypeHash = <Test as frame_system::Config>::Hash;
pub type TestDidIdentifier = <Test as did::Config>::DidIdentifier;

pub(crate) const DEFAULT_ACCOUNT: AccountId = AccountId::new([0u8; 32]);

pub struct ExtBuilder {
	did_builder: Option<did_mock::ExtBuilder>,
	ctypes_stored: Vec<(TestCtypeHash, TestDidIdentifier)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			did_builder: None,
			ctypes_stored: vec![],
		}
	}
}

impl From<did_mock::ExtBuilder> for ExtBuilder {
    fn from(did_builder: did_mock::ExtBuilder) -> Self {
		Self {
			did_builder: Some(did_builder),
			ctypes_stored: vec![]
		}
    }
}

impl ExtBuilder {
	pub fn with_ctypes(mut self, ctypes: Vec<(TestCtypeHash, TestDidIdentifier)>) -> Self {
		self.ctypes_stored = ctypes;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut ext = if let Some(did_builder) = self.did_builder.clone() {
			did_builder.build()
		} else {
			let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			sp_io::TestExternalities::new(storage)
		};

		if self.ctypes_stored.len() > 0 {
			ext.execute_with(|| {
				self.ctypes_stored.iter().for_each(|ctype| {
					ctype::CTYPEs::<Test>::insert(ctype.0.clone(), ctype.1.clone());
				})
			});
		}

		ext
	}
}
