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

use codec::MaxEncodedLen;
use frame_support::{traits::Currency, BoundedVec};

use pallet_did_lookup::associate_account_request::AssociateAccountRequest;
use runtime_common::constants::{
	attestation::MAX_ATTESTATION_BYTE_LENGTH, did::MAX_DID_BYTE_LENGTH, did_lookup::MAX_CONNECTION_BYTE_LENGTH,
	web3_names::MAX_NAME_BYTE_LENGTH, MAX_INDICES_BYTE_LENGTH,
};

#[cfg(test)]
use runtime_common::{AccountId, BlockNumber};

use super::{Call, Runtime};

#[test]
fn call_size() {
	assert!(
		core::mem::size_of::<Call>() <= 240,
		"size of Call is more than 240 bytes: some calls have too big arguments, use Box to reduce \
		the size of Call.
		If the limit is too strong, maybe consider increase the limit to 300.",
	);
}
