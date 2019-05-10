
//! Error: Handles errors for all other runtime modules

use support::{decl_event, decl_module, Parameter };
use runtime_primitives::traits::{ SimpleArithmetic, Member, MaybeDisplay, MaybeSerializeDebug, Bounded, As };

/// The error trait
pub trait Trait: system::Trait {
    type ErrorCode : Parameter + Member + MaybeSerializeDebug + MaybeDisplay + SimpleArithmetic + Bounded;
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
        Self::deposit_event(RawEvent::ErrorOccurred(T::ErrorCode::sa(error_type.0.into())));
        return Err(error_type.1);
    }
}
