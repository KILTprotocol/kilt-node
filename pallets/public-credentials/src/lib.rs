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

//! # Public credentials Pallet
//!
//! Provides means of issuing public KILT credentials on chain and revoking
//! them.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod assets;
pub mod credentials;
pub mod default_weights;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::MaxEncodedLen;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{ConstU32, Get, StorageVersion},
		Blake2_128Concat, Twox64Concat,
	};
	use frame_system::pallet_prelude::{BlockNumberFor, *};
	use sp_core::H256;

	use crate::credentials::{Claim, Credential};
	use attestation::{Attestations, AttesterOf, ClaimHashOf};
	use ctype::{CtypeHashOf, Ctypes};
	use kilt_support::{signature::VerifySignature, traits::CallSources};

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	// TODO: Replace with an enum that includes KILT DIDs and asset DIDs.
	pub(crate) type SubjectIdOf<T> = AccountIdOf<T>;

	pub type CredentialOf<T> = Credential<
		CtypeHashOf<T>,
		BoundedVec<u8, ConstU32<10>>,
		BoundedVec<u8, <T as Config>::MaxEncodedClaimContentLength>,
		<T as Config>::CredentialSignature,
		ClaimHashOf<T>,
		H256,
	>;

	#[pallet::config]
	pub trait Config: frame_system::Config + attestation::Config {
		type EnsureOrigin: EnsureOrigin<
			Success = <Self as Config>::OriginSuccess,
			<Self as frame_system::Config>::Origin,
		>;
		type CredentialSignatureVerification: VerifySignature<
			SignerId = Self::CredentialClaimerIdentifier,
			Payload = ClaimHashOf<Self>,
			Signature = Self::CredentialSignature,
		>;
		type CredentialSignature: Parameter;
		type CredentialClaimerIdentifier: Parameter + MaxEncodedLen;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type MaxEncodedClaimContentLength: Get<u32>;
		type OriginSuccess: CallSources<AccountIdOf<Self>, AttesterOf<Self>>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_credential_info)]
	pub type Credentials<T> =
		StorageDoubleMap<_, Twox64Concat, SubjectIdOf<T>, Blake2_128Concat, ClaimHashOf<T>, BlockNumberFor<T>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Test,
	}

	#[pallet::error]
	pub enum Error<T> {
		Test,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn add(origin: OriginFor<T>, credential: CredentialOf<T>) -> DispatchResult {
			let source = <T as Config>::EnsureOrigin::ensure_origin(origin)?;
			let attester = source.subject();
			let payer = source.sender();

			let Credential {
				claim: Claim {
					ctype_hash,
					subject,
					contents,
				},
				claimer_signature,
				nonce,
				claim_hash,
			} = credential;

			// Check that a CType exists.
			ensure!(
				Ctypes::<T>::contains_key(&credential.claim.ctype_hash),
				// FIXME
				Error::<T>::Test
			);

			// Check that an attestation with the same hash does NOT exist.
			ensure!(
				!Attestations::<T>::contains_key(&credential.claim_hash),
				// FIXME
				Error::<T>::Test
			);

			Ok(())
		}
	}
}
