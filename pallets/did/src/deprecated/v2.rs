use codec::{Decode, Encode};
use kilt_primitives::Hash;

pub(crate) use super::*;
use crate::*;

#[derive(Clone, Decode, Encode, PartialEq, Eq)]
pub(crate) enum ContentType {
	ApplicationJson,
	ApplicationJsonLd,
}

#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DidDetails<T: Config> {
	pub(crate) authentication_key: KeyIdOf<T>,
	pub(crate) key_agreement_keys: DidKeyAgreementKeySet<T>,
	pub(crate) delegation_key: Option<KeyIdOf<T>>,
	pub(crate) attestation_key: Option<KeyIdOf<T>>,
	pub(crate) public_keys: DidPublicKeyMap<T>,
	pub(crate) service_endpoints: Option<ServiceEndpoints>,
	pub(crate) last_tx_counter: u64,
}

#[derive(Clone, Decode, Encode, PartialEq)]
pub(crate) struct ServiceEndpoints {
	pub content_hash: Hash,
	pub urls: Vec<Url>,
	pub content_type: ContentType,
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
