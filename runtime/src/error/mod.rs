// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019  BOTLabs GmbH

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

//! Error: Handles errors for all other runtime modules

use runtime_primitives::traits::{
	As, Bounded, MaybeDisplay, MaybeSerializeDebug, Member, SimpleArithmetic,
};
use support::{decl_event, decl_module, Parameter};

/// The error trait
pub trait Trait: system::Trait {
	type ErrorCode: Parameter + Member + MaybeSerializeDebug + MaybeDisplay + SimpleArithmetic + Bounded;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// The error type is a tuple of error code and an error message
pub type ErrorType = (u16, &'static str);

decl_event!(
	/// Events for errors
	pub enum Event<T> where <T as Trait>::ErrorCode {
		// An error occurred
		ErrorOccurred(ErrorCode),
	}
);

decl_module! {
	/// The error runtime module. Since it is used by other modules to deposit events, it has no transaction functions.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		/// Deposit events
		fn deposit_event<T>() = default;

	}
}

/// Implementation of further module functions for errors
impl<T: Trait> Module<T> {
	/// Create an error, it logs the error, deposits an error event and returns the error with its message
	pub fn error(error_type: ErrorType) -> Result<(), &'static str> {
		::runtime_io::print(error_type.1);
		Self::deposit_event(RawEvent::ErrorOccurred(T::ErrorCode::sa(
			error_type.0.into(),
		)));
		return Err(error_type.1);
	}

	/// Create an error, it logs the error, deposits an error event and returns the error message
	pub fn deposit_err(error_type: ErrorType) -> &'static str {
		::runtime_io::print(error_type.1);
		Self::deposit_event(RawEvent::ErrorOccurred(T::ErrorCode::sa(
			error_type.0.into(),
		)));
		error_type.1
	}

	pub fn ok_or_deposit_err<S>(opt: Option<S>, error_type: ErrorType) -> Result<S, &'static str> {
		if let Some(s) = opt {
			Ok(s)
		} else {
			Err(Self::deposit_err(error_type))
		}
	}
}
