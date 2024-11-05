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
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[derive(Encode, Decode, TypeInfo)]
pub struct AddressResult<Address, Extra> {
	address: Address,
	extra: Option<Extra>,
}

impl<Address, Extra> AddressResult<Address, Extra> {
	pub const fn new(address: Address, extra: Option<Extra>) -> Self {
		Self { address, extra }
	}
}

#[derive(Encode, Decode, TypeInfo)]
pub struct NameResult<Name, Extra> {
	name: Name,
	extra: Option<Extra>,
}

impl<Name, Extra> NameResult<Name, Extra> {
	pub const fn new(name: Name, extra: Option<Extra>) -> Self {
		Self { name, extra }
	}
}

sp_api::decl_runtime_apis! {
	pub trait UniqueLookup<Address, Name, Extra, Error> where
		Address: Codec,
		Name: Codec,
		Extra: Codec,
		Error: Codec,
		{
			fn address_for_name(name: Name) -> Result<Option<AddressResult<Address, Extra>>, Error>;
			fn batch_address_for_name(names: Vec<Name>) -> Result<Vec<Option<AddressResult<Address, Extra>>>, Error>;
			fn name_for_address(address: Address) -> Result<Option<NameResult<Name, Extra>>, Error>;
			fn batch_name_for_address(addresses: Vec<Address>) -> Result<Vec<Option<NameResult<Name, Extra>>>, Error>;
		}
}
