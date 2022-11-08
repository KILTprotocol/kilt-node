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

use kilt_support::traits::ItemFilter;

sp_api::decl_runtime_apis! {
	/// The API to query public credentials for a subject.
	pub trait PublicCredentials<SubjectId, CredentialId, CredentialEntry, Filter, Error> where
		SubjectId: Codec,
		CredentialId: Codec,
		CredentialEntry: Codec,
		Filter: Codec + ItemFilter<CredentialEntry>,
		Error: Codec,
	{
		/// Return the public credential with the specified ID, if found.
		fn get_by_id(credential_id: CredentialId) -> Option<CredentialEntry>;
		/// Return all the public credentials linked to the specified subject.
		/// An optional filter can be passed to be applied to the result before being returned to the client.
		/// It returns an error if the provided specified subject ID is not valid.
		fn get_by_subject(subject: SubjectId, filter: Option<Filter>) -> Result<Vec<(CredentialId, CredentialEntry)>, Error>;
	}
}
