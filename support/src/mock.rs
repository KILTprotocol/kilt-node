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

///! This module contains utilities for testing.
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::sr25519;
use sp_runtime::AccountId32;

/// This pallet only contains an origin which supports separated sender and
/// subject.
///
/// WARNING: This is only used for testing!
#[frame_support::pallet]
#[allow(dead_code)]
pub mod mock_origin {
	use sp_std::marker::PhantomData;

	use codec::{Decode, Encode};
	use frame_support::{traits::EnsureOrigin, Parameter};
	use scale_info::TypeInfo;

	use crate::traits::CallSources;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Origin: From<Origin<Self>>;
		type AccountId: Parameter + Default;
		type SubjectId: Parameter + Default;
	}

	/// A dummy pallet for adding an origin to the runtime that contains
	/// separate sender and subject accounts.
	///
	/// WARNING: This is only used for testing!
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// An origin that is split into sender and subject.
	///
	/// WARNING: This is only used for testing!
	#[pallet::origin]
	pub type Origin<T> = DoubleOrigin<<T as Config>::AccountId, <T as Config>::SubjectId>;

	/// An origin that is split into sender and subject.
	///
	/// WARNING: This is only used for testing!
	#[derive(Debug, Clone, Default, PartialEq, Eq, TypeInfo, Encode, Decode)]
	pub struct DoubleOrigin<AccountId, SubjectId>(pub AccountId, pub SubjectId);

	impl<AccountId: Clone + Default, SubjectId: Clone + Default> CallSources<AccountId, SubjectId>
		for DoubleOrigin<AccountId, SubjectId>
	{
		fn sender(&self) -> AccountId {
			self.0.clone()
		}

		fn subject(&self) -> SubjectId {
			self.1.clone()
		}
	}

	/// Ensure that the call was made using the split origin.
	///
	/// WARNING: This is only used for testing!
	pub struct EnsureDoubleOrigin<AccountId, SubjectId>(PhantomData<(AccountId, SubjectId)>);

	impl<OuterOrigin, AccountId: Default, SubjectId: Default> EnsureOrigin<OuterOrigin>
		for EnsureDoubleOrigin<AccountId, SubjectId>
	where
		OuterOrigin:
			Into<Result<DoubleOrigin<AccountId, SubjectId>, OuterOrigin>> + From<DoubleOrigin<AccountId, SubjectId>>,
	{
		type Success = DoubleOrigin<AccountId, SubjectId>;

		fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
			o.into()
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn successful_origin() -> OuterOrigin {
			OuterOrigin::from(Default::default())
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<OuterOrigin, AccountId, SubjectId> crate::traits::GenerateBenchmarkOrigin<OuterOrigin, AccountId, SubjectId>
		for EnsureDoubleOrigin<AccountId, SubjectId>
	where
		OuterOrigin:
			Into<Result<DoubleOrigin<AccountId, SubjectId>, OuterOrigin>> + From<DoubleOrigin<AccountId, SubjectId>>,
	{
		fn generate_origin(sender: AccountId, subject: SubjectId) -> OuterOrigin {
			OuterOrigin::from(DoubleOrigin(sender, subject))
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Default)]
pub struct SubjectId(pub AccountId32);

impl From<AccountId32> for SubjectId {
	fn from(acc: AccountId32) -> Self {
		SubjectId(acc)
	}
}

impl From<sr25519::Public> for SubjectId {
	fn from(acc: sr25519::Public) -> Self {
		SubjectId(acc.into())
	}
}
