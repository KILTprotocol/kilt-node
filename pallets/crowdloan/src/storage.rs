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
use scale_info::TypeInfo;
use sp_runtime::Permill;

/// A set of reserve accounts
#[derive(Clone, Debug, Default, Decode, PartialEq, Encode, TypeInfo)]
pub struct ReserveAccounts<A: Default> {
	/// The account that is used to do vested transfers.
	pub vested: A,
	/// The account that is used to do free and unlocked transfers.
	pub free: A,
}

/// The configuration of the gratitude.
#[derive(Clone, Debug, Default, Decode, Encode, PartialEq, TypeInfo)]
pub struct GratitudeConfig<BlockNumber: Default> {
	/// The permill of vested tokens that are given.
	pub vested_share: Permill,
	/// The start block of the vesting.
	pub start_block: BlockNumber,
	/// The length of the vesting.
	pub vesting_length: BlockNumber,
}
