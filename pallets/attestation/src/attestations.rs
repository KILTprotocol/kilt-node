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

use codec::{Decode, Encode};
use ctype::CtypeHashOf;
use delegation::DelegationNodeIdOf;
use kilt_support::deposit::Deposit;
use scale_info::TypeInfo;

use crate::{AccountIdOf, AttesterOf, BalanceOf, Config};

/// An on-chain attestation written by an attester.
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct AttestationDetails<T: Config> {
	/// The hash of the CType used for this attestation.
	pub ctype_hash: CtypeHashOf<T>,
	/// The ID of the attester.
	pub attester: AttesterOf<T>,
	/// \[OPTIONAL\] The ID of the delegation node used to authorize the
	/// attester.
	pub delegation_id: Option<DelegationNodeIdOf<T>>,
	/// The flag indicating whether the attestation has been revoked or not.
	pub revoked: bool,
	/// The deposit that was taken to incentivise fair use of the on chain
	/// storage.
	pub deposit: Deposit<AccountIdOf<T>, BalanceOf<T>>,
}
