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

use frame_support::sp_runtime::RuntimeDebug;

// Only re-export the main enum, with the variant values still being namespaced
pub use asset::Error as AssetError;
pub use chain::Error as ChainError;

/// An error in the Asset DID parsing logic.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
pub enum AssetDidError {
	/// An error in the chain ID parsing logic.
	ChainId(ChainError),
	/// An error in the asset ID parsing logic.
	AssetId(AssetError),
	/// A generic error not belonging to any of the other categories.
	InvalidFormat,
}

impl From<ChainError> for AssetDidError {
	fn from(err: ChainError) -> Self {
		Self::ChainId(err)
	}
}

impl From<AssetError> for AssetDidError {
	fn from(err: AssetError) -> Self {
		Self::AssetId(err)
	}
}

pub mod chain {
	use super::*;

	/// An error in the chain ID parsing logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
	pub enum Error {
		/// An error in the chain namespace parsing logic.
		Namespace(NamespaceError),
		/// An error in the chain reference parsing logic.
		Reference(ReferenceError),
		/// A generic error not belonging to any of the other categories.
		InvalidFormat,
	}

	/// An error in the chain namespace parsing logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
	pub enum NamespaceError {
		/// Namespace too long.
		TooLong,
		/// Namespace too short.
		TooShort,
		/// A generic error not belonging to any of the other categories.
		InvalidFormat,
	}

	/// An error in the chain reference parsing logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
	pub enum ReferenceError {
		/// Reference too long.
		TooLong,
		/// Reference too short.
		TooShort,
		/// A generic error not belonging to any of the other categories.
		InvalidFormat,
	}

	impl From<NamespaceError> for Error {
		fn from(err: NamespaceError) -> Self {
			Self::Namespace(err)
		}
	}

	impl From<ReferenceError> for Error {
		fn from(err: ReferenceError) -> Self {
			Self::Reference(err)
		}
	}
}

pub mod asset {
	use super::*;

	/// An error in the asset ID parsing logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
	pub enum Error {
		/// An error in the asset namespace parsing logic.
		Namespace(NamespaceError),
		/// An error in the asset reference parsing logic.
		Reference(ReferenceError),
		/// An error in the asset identifier parsing logic.
		Identifier(IdentifierError),
		/// A generic error not belonging to any of the other categories.
		InvalidFormat,
	}

	/// An error in the asset namespace parsing logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
	pub enum NamespaceError {
		/// Namespace too long.
		TooLong,
		/// Namespace too short.
		TooShort,
		/// A generic error not belonging to any of the other categories.
		InvalidFormat,
	}

	/// An error in the asset reference parsing logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
	pub enum ReferenceError {
		/// Reference too long.
		TooLong,
		/// Reference too short.
		TooShort,
		/// A generic error not belonging to any of the other categories.
		InvalidFormat,
	}

	/// An error in the asset identifier parsing logic.
	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug)]
	pub enum IdentifierError {
		/// Identifier too long.
		TooLong,
		/// Identifier too short.
		TooShort,
		/// A generic error not belonging to any of the other categories.
		InvalidFormat,
	}

	impl From<NamespaceError> for Error {
		fn from(err: NamespaceError) -> Self {
			Self::Namespace(err)
		}
	}

	impl From<ReferenceError> for Error {
		fn from(err: ReferenceError) -> Self {
			Self::Reference(err)
		}
	}

	impl From<IdentifierError> for Error {
		fn from(err: IdentifierError) -> Self {
			Self::Identifier(err)
		}
	}
}
