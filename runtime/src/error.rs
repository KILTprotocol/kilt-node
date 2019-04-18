
use support::{decl_event, decl_module, Parameter };
use runtime_primitives::traits::{ SimpleArithmetic, Member, MaybeDisplay, MaybeSerializeDebug, Bounded, As };

decl_event!(
	pub enum Event<T> where <T as Trait>::ErrorCode {
        // An error occurred
		ErrorOccurred(ErrorCode),
	}
);

pub trait Trait: system::Trait {
    type ErrorCode : Parameter + Member + MaybeSerializeDebug + MaybeDisplay + SimpleArithmetic + Bounded;
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type ErrorType = (u16, &'static str);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

	}
}

impl<T: Trait> Module<T> {
    pub fn error(error_type: ErrorType) -> Result<(), &'static str> {
        ::runtime_io::print(error_type.1);
        Self::deposit_event(RawEvent::ErrorOccurred(T::ErrorCode::sa(error_type.0.into())));
        return Err(error_type.1);
    }
}
