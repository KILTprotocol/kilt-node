// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use frame_support::traits::{EnsureOrigin, EnsureOriginWithArg};
use kilt_support::traits::CallSources;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_runtime::RuntimeDebug;
use sp_std::marker::PhantomData;

/// Origin for modules that support DID-based authorization.
#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DidRawOrigin<DidIdentifier, AccountId> {
	pub id: DidIdentifier,
	pub submitter: AccountId,
}

impl<DidIdentifier, AccountId> DidRawOrigin<DidIdentifier, AccountId> {
	pub const fn new(id: DidIdentifier, submitter: AccountId) -> Self {
		Self { id, submitter }
	}
}

impl<DidIdentifier: Clone, AccountId: Clone> CallSources<AccountId, DidIdentifier>
	for DidRawOrigin<DidIdentifier, AccountId>
{
	fn sender(&self) -> AccountId {
		self.submitter.clone()
	}

	fn subject(&self) -> DidIdentifier {
		self.id.clone()
	}
}

pub struct EnsureDidOrigin<DidIdentifier, AccountId, ExpectedSubmitter = ()>(
	PhantomData<(DidIdentifier, AccountId, ExpectedSubmitter)>,
);

impl<OuterOrigin, DidIdentifier, AccountId, ExpectedSubmitter> EnsureOrigin<OuterOrigin>
	for EnsureDidOrigin<DidIdentifier, AccountId, ExpectedSubmitter>
where
	OuterOrigin: Into<Result<DidRawOrigin<DidIdentifier, AccountId>, OuterOrigin>>
		+ From<DidRawOrigin<DidIdentifier, AccountId>>
		+ Clone,
	DidIdentifier: From<AccountId>,
	AccountId: Clone + Decode + PartialEq,
	ExpectedSubmitter: Get<Option<AccountId>>,
{
	type Success = DidRawOrigin<DidIdentifier, AccountId>;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		let did_raw_origin = o.clone().into()?;
		// Origin check succeeds if no authorized account is configured, or if the one
		// configured matches the tx submitter. Fails otherwise.
		match ExpectedSubmitter::get() {
			None => Ok(did_raw_origin),
			Some(authorised_submitter) if authorised_submitter == did_raw_origin.submitter => Ok(did_raw_origin),
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<OuterOrigin, ()> {
		let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
			.expect("infinite length input; no invalid inputs for type; qed");

		Ok(OuterOrigin::from(DidRawOrigin {
			id: zero_account_id.clone().into(),
			submitter: zero_account_id,
		}))
	}
}

impl<OuterOrigin, DidIdentifier, AccountId, ExpectedSubmitter> EnsureOriginWithArg<OuterOrigin, DidIdentifier>
	for EnsureDidOrigin<DidIdentifier, AccountId, ExpectedSubmitter>
where
	OuterOrigin: Into<Result<DidRawOrigin<DidIdentifier, AccountId>, OuterOrigin>>
		+ From<DidRawOrigin<DidIdentifier, AccountId>>
		+ Clone,
	DidIdentifier: PartialEq<DidIdentifier> + Clone,
	AccountId: Clone + Decode,
{
	type Success = DidRawOrigin<DidIdentifier, AccountId>;

	fn try_origin(o: OuterOrigin, a: &DidIdentifier) -> Result<Self::Success, OuterOrigin> {
		let did_origin: DidRawOrigin<DidIdentifier, AccountId> = o.clone().into()?;
		if did_origin.id == *a {
			Ok(did_origin)
		} else {
			Err(o)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(a: &DidIdentifier) -> Result<OuterOrigin, ()> {
		let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
			.expect("infinite length input; no invalid inputs for type; qed");

		Ok(OuterOrigin::from(DidRawOrigin {
			id: a.clone(),
			submitter: zero_account_id,
		}))
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<OuterOrigin, AccountId, DidIdentifier, ExpectedSubmitter>
	kilt_support::traits::GenerateBenchmarkOrigin<OuterOrigin, AccountId, DidIdentifier>
	for EnsureDidOrigin<DidIdentifier, AccountId, ExpectedSubmitter>
where
	OuterOrigin: Into<Result<DidRawOrigin<DidIdentifier, AccountId>, OuterOrigin>>
		+ From<DidRawOrigin<DidIdentifier, AccountId>>,
{
	fn generate_origin(sender: AccountId, subject: DidIdentifier) -> OuterOrigin {
		OuterOrigin::from(DidRawOrigin {
			id: subject,
			submitter: sender,
		})
	}
}

#[cfg(test)]
mod tests {
	#[cfg(feature = "runtime-benchmarks")]
	#[test]
	pub fn successful_origin() {
		use crate::{
			mock::{AccountId, DidIdentifier, Test},
			EnsureDidOrigin,
		};
		use frame_support::{assert_ok, traits::EnsureOrigin};

		let origin: <Test as frame_system::Config>::RuntimeOrigin =
			EnsureDidOrigin::<DidIdentifier, AccountId>::try_successful_origin()
				.expect("Successful origin creation should not fail.");
		assert_ok!(EnsureDidOrigin::<DidIdentifier, AccountId>::try_origin(origin));
	}
}
