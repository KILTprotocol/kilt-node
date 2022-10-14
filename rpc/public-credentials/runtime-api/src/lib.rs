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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	/// The API to query public credentials for a subject.
	pub trait PublicCredentialsApi<SubjectId, CredentialId, CredentialEntry> where
		SubjectId: Codec,
		CredentialId: Codec,
		CredentialEntry: Codec
	{
		fn get_credential(credential_id: CredentialId) -> Option<CredentialEntry>;
		fn get_credentials(subject: SubjectId) -> Vec<(CredentialId, CredentialEntry)>;
	}
}
