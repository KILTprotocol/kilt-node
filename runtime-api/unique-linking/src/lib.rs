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

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Codec, Decode, Encode};
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub struct AddressResult<Address, Extra> {
	address: Address,
	extra: Option<Extra>,
}

#[derive(Encode, Decode)]
pub struct NameResult<Name, Extra> {
	name: Name,
	extra: Option<Extra>,
}

sp_api::decl_runtime_apis! {
	pub trait UniqueLookup<Address, Name, Extra> where
		Address: Codec,
		Name: Codec,
		Extra: Codec,
		{
			fn address_for_name(name: Name) -> Option<AddressResult<Address, Extra>>;
			fn batch_address_for_name(names: Vec<Name>) -> Vec<Option<AddressResult<Address, Extra>>>;
			fn name_for_address(address: Address) -> Option<NameResult<Name, Extra>>;
			fn batch_name_for_address(addresses: Vec<Address>) -> Vec<Option<NameResult<Name, Extra>>>;
		}
}
