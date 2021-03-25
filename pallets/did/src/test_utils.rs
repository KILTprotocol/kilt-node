use crate::*;
use frame_support::{dispatch::Weight, weights::constants::WEIGHT_PER_SECOND};
use sp_runtime::Perbill;
use tests::*;

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

// A test DID operation which can be crated to require any dUD verification key type.
pub struct TestDIDOperation {
	pub did: DIDIdentifier,
	pub verification_key_type: DIDVerificationKeyType,
}

impl DIDOperation for TestDIDOperation {
	fn get_verification_key_type(&self) -> DIDVerificationKeyType {
		self.verification_key_type.clone()
	}

	fn get_did(&self) -> &DIDIdentifier {
		&self.did
	}
}

impl Encode for TestDIDOperation {
	fn size_hint(&self) -> usize {
		100
	}

	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.using_encoded(|buf| dest.write(buf));
	}

	fn encode(&self) -> Vec<u8> {
		let mut r = Vec::with_capacity(self.size_hint());
		self.encode_to(&mut r);
		r
	}

	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		f([1u8; 100].as_ref())
	}

	fn encoded_size(&self) -> usize {
		100
	}
}
