use codec::{Decode, Encode};

pub(crate) use super::*;
use crate::*;

#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DidDetails<T: Config> {
	pub(crate) authentication_key: KeyIdOf<T>,
	pub(crate) key_agreement_keys: DidKeyAgreementKeySet<T>,
	pub(crate) delegation_key: Option<KeyIdOf<T>>,
	pub(crate) attestation_key: Option<KeyIdOf<T>>,
	pub(crate) public_keys: DidPublicKeyMap<T>,
	pub(crate) endpoint_url: Option<Url>,
	pub(crate) last_tx_counter: u64,
}

#[cfg(test)]
impl<T: Config> DidDetails<T> {
	pub(crate) fn new(authentication_key: DidVerificationKey, block_number: BlockNumberOf<T>) -> Self {
		let mut public_keys = DidPublicKeyMap::<T>::default();
		let authentication_key_id = utils::calculate_key_id::<T>(&authentication_key.clone().into());
		public_keys.try_insert(
			authentication_key_id,
			DidPublicKeyDetails {
				key: authentication_key.into(),
				block_number,
			},
		).unwrap();
		Self {
			authentication_key: authentication_key_id,
			key_agreement_keys: DidKeyAgreementKeySet::<T>::default(),
			attestation_key: None,
			delegation_key: None,
			endpoint_url: None,
			public_keys,
			last_tx_counter: 0u64,
		}
	}
}

pub(crate) mod storage {
	use frame_support::{decl_module, decl_storage};
	use sp_std::prelude::*;

	use super::*;

	decl_module! {
		pub struct OldPallet<T: Config> for enum Call where origin: <T as pallet::Config>::Origin {}
	}

	decl_storage! {
		trait Store for OldPallet<T: Config> as Did {
			pub(crate) Did get(fn did): map hasher(blake2_128_concat) DidIdentifierOf<T> => Option<super::DidDetails<T>>;
		}
	}
}
