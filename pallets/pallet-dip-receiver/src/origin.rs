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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::EnsureOrigin, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DipOrigin<Identifier, AccountId, Proof> {
	pub identifier: Identifier,
	pub account_address: AccountId,
	pub proof: Proof,
}

pub struct EnsureDipOrigin<Identifier, AccountId, Proof>(PhantomData<(Identifier, AccountId, Proof)>);

#[cfg(not(feature = "runtime-benchmarks"))]
impl<OuterOrigin, Identifier, AccountId, Proof> EnsureOrigin<OuterOrigin>
	for EnsureDipOrigin<Identifier, AccountId, Proof>
where
	OuterOrigin: From<DipOrigin<Identifier, AccountId, Proof>>
		+ Into<Result<DipOrigin<Identifier, AccountId, Proof>, OuterOrigin>>,
{
	type Success = DipOrigin<Identifier, AccountId, Proof>;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		o.into()
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<OuterOrigin, Identifier, AccountId, Proof> EnsureOrigin<OuterOrigin>
	for EnsureDipOrigin<Identifier, AccountId, Proof>
where
	OuterOrigin: From<DipOrigin<Identifier, AccountId, Proof>>
		+ Into<Result<DipOrigin<Identifier, AccountId, Proof>, OuterOrigin>>,
	// Additional trait bounds only valid when benchmarking
	Identifier: From<[u8; 32]>,
	AccountId: From<[u8; 32]>,
	Proof: Default,
{
	type Success = DipOrigin<Identifier, AccountId, Proof>;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		o.into()
	}

	fn try_successful_origin() -> Result<OuterOrigin, ()> {
		Ok(OuterOrigin::from(DipOrigin {
			identifier: Identifier::from([0u8; 32]),
			account_address: AccountId::from([0u8; 32]),
			proof: Default::default(),
		}))
	}
}

impl<Identifier, AccountId, Proof> kilt_support::traits::CallSources<AccountId, Identifier>
	for DipOrigin<Identifier, AccountId, Proof>
where
	Identifier: Clone,
	AccountId: Clone,
{
	fn sender(&self) -> AccountId {
		self.account_address.clone()
	}

	fn subject(&self) -> Identifier {
		self.identifier.clone()
	}
}
