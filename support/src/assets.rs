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

pub use account_id::*;
pub use asset_id::*;
pub use chain_id::*;

pub mod chain_id {
	use core::str::FromStr;

	const NAMESPACE_MIN_LEN: usize = 3;
	const NAMESPACE_MAX_LEN: usize = 8;
	const REFERENCE_MIN_LEN: usize = 1;
	const REFERENCE_MAX_LEN: usize = 32;

	pub struct Namespace(Vec<u8>);
	pub struct Reference(Vec<u8>);

	pub enum ChainIdentifier {
		Eip155(Eip155Namespace),
		Bip122(Bip122Namespace),
		Polkadot(PolkadotNamespace),
		Solana(SolanaNamespace),
	}

	pub enum Eip155Namespace {
		Mainnet,
		Moonbeam,
		Moonriver,
		Other(Reference),
	}

	pub enum Bip122Namespace {
		Bitcoin,
		Other(Reference),
	}

	pub enum PolkadotNamespace {
		Polkadot,
		Kusama,
		Spiritnet,
		Other(Reference),
	}

	pub enum SolanaNamespace {
		Mainnet,
	}
}

mod account_id {
	use crate::assets::ChainIdentifier;

	// [a-zA-Z0-9]{1,64}
	pub type AccountId = [u8; 64];

	pub enum AccountIdentifier {}
}

mod asset_id {}
