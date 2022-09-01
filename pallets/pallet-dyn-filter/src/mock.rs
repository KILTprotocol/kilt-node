// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use frame_support::{parameter_types, traits::Contains};
use frame_system::EnsureRoot;
use lazy_static::lazy_static;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};

pub(crate) type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub(crate) type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type Hash = sp_core::H256;
pub(crate) type Signature = MultiSignature;
pub(crate) type AccountPublic = <Signature as Verify>::Signer;
pub(crate) type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub(crate) type Index = u64;
pub(crate) type BlockNumber = u64;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		DynFilter: crate::{Pallet, Storage, Call, Event<T>},
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: BlockNumber = 2400;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl crate::Config for Test {
	type Event = Event;
	type WeightInfo = ();

	type ApproveOrigin = EnsureRoot<AccountId>;
	type FeatureCall = FeatureCalls;
	type TransferCall = TransferCalls;
	type XcmCall = XcmCalls;
	type SystemCall = SystemCalls;
}

const TRANSFER: &[u8] = b"trf";
const FEATURE: &[u8] = b"fet";
const XCM: &[u8] = b"xcm";
const SYSTEM: &[u8] = b"system";

lazy_static! {
	pub static ref CALL_TRANSFER: Call = Call::System(frame_system::Call::remark {
		remark: TRANSFER.to_vec()
	});
	pub static ref CALL_FEATURE: Call = Call::System(frame_system::Call::remark {
		remark: FEATURE.to_vec()
	});
	pub static ref CALL_XCM: Call = Call::System(frame_system::Call::remark { remark: XCM.to_vec() });
	pub static ref CALL_SYSTEM: Call = Call::System(frame_system::Call::remark {
		remark: SYSTEM.to_vec()
	});
}

pub struct TransferCalls;
impl Contains<Call> for TransferCalls {
	fn contains(t: &Call) -> bool {
		if let Call::System(frame_system::Call::remark { remark }) = t {
			&remark[..] == TRANSFER
		} else {
			false
		}
	}
}

pub struct FeatureCalls;
impl Contains<Call> for FeatureCalls {
	fn contains(t: &Call) -> bool {
		if let Call::System(frame_system::Call::remark { remark }) = t {
			&remark[..] == FEATURE
		} else {
			false
		}
	}
}
pub struct XcmCalls;
impl Contains<Call> for XcmCalls {
	fn contains(t: &Call) -> bool {
		if let Call::System(frame_system::Call::remark { remark }) = t {
			&remark[..] == XCM
		} else {
			false
		}
	}
}

pub struct SystemCalls;
impl Contains<Call> for SystemCalls {
	fn contains(t: &Call) -> bool {
		if let Call::System(frame_system::Call::remark { remark }) = t {
			&remark[..] == SYSTEM
		} else {
			false
		}
	}
}

#[derive(Clone, Default)]
pub struct ExtBuilder {}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		sp_io::TestExternalities::new(storage)
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();

		let keystore = sp_keystore::testing::KeyStore::new();
		ext.register_extension(sp_keystore::KeystoreExt(std::sync::Arc::new(keystore)));

		ext
	}
}
