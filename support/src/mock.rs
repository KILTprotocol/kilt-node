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
		type DidIdentifier: Parameter + Default;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::origin]
	pub type Origin<T> = DoubleOrigin<<T as Config>::AccountId, <T as Config>::DidIdentifier>;

	#[derive(Debug, Clone, Default, PartialEq, Eq, TypeInfo, Encode, Decode)]
	pub struct DoubleOrigin<AccountId, DidIdentifier>(pub AccountId, pub DidIdentifier);

	impl<AccountId: Clone + Default, DidIdentifier: Clone + Default> CallSources<AccountId, DidIdentifier> for DoubleOrigin<AccountId, DidIdentifier> {
		fn sender(&self) -> AccountId {
			self.0.clone()
		}

		fn subject(&self) -> DidIdentifier {
			self.1.clone()
		}
	}

	pub struct EnsureDoubleOrigin<AccountId, DidIdentifier>(PhantomData<(AccountId, DidIdentifier)>);

	impl<OuterOrigin, AccountId: Default, DidIdentifier: Default> EnsureOrigin<OuterOrigin> for EnsureDoubleOrigin<AccountId, DidIdentifier>
	where
		OuterOrigin: Into<Result<DoubleOrigin<AccountId, DidIdentifier>, OuterOrigin>>
			+ From<DoubleOrigin<AccountId, DidIdentifier>>,
	{
		type Success = DoubleOrigin<AccountId, DidIdentifier>;

		fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
			o.into()
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn successful_origin() -> OuterOrigin {
			// don't use
			OuterOrigin::from(Default::default())
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<OuterOrigin, AccountId, DidIdentifier>
		crate::traits::GenerateBenchmarkOrigin<OuterOrigin, AccountId, DidIdentifier>
		for EnsureDoubleOrigin<AccountId, DidIdentifier>
	where
		OuterOrigin: Into<Result<DoubleOrigin<AccountId, DidIdentifier>, OuterOrigin>>
			+ From<DoubleOrigin<AccountId, DidIdentifier>>,
	{
		fn generate_origin(sender: AccountId, subject: DidIdentifier) -> OuterOrigin {
			OuterOrigin::from(DoubleOrigin(sender, subject))
		}
	}
}
