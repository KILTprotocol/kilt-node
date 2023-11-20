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

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::Codec;

sp_api::decl_runtime_apis! {
	/// Runtime API to generate a DIP proof with the provided parameters.
	pub trait DipProvider<ProofRequest, Success, Error> where
		ProofRequest: Codec,
		Success: Codec,
		Error: Codec,
		{
			/// Generate a DIP proof with the parameters specified in the request.
			fn generate_proof(request: ProofRequest) -> Result<Success, Error>;
		}
}
