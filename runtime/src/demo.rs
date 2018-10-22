// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)

use parity_codec::Encode;
use srml_support::{StorageValue, dispatch::Result};
use runtime_primitives::traits::{Hash, OnFinalise};
use {balances, system::{self, ensure_signed}};

pub trait Trait: balances::Trait {}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn play(origin) -> Result;
		fn set_payment(origin, value: T::Balance) -> Result;
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Demo {
		Payment get(payment) config(): Option<T::Balance>;
		Pot get(pot): T::Balance;
	}
}

impl<T: Trait> Module<T> {
	fn play(origin: T::Origin) -> Result {
		let sender = ensure_signed(origin)?;
		let payment = Self::payment().ok_or("Must have payment amount set")?;

		<balances::Module<T>>::decrease_free_balance(&sender, payment)?;

		if (<system::Module<T>>::random_seed(), &sender)
			.using_encoded(<T as system::Trait>::Hashing::hash)
			.using_encoded(|e| e[0] < 128)
		{
			<balances::Module<T>>::increase_free_balance_creating(&sender, <Pot<T>>::take());
		}

		<Pot<T>>::mutate(|pot| *pot += payment);

		Ok(())
	}

	fn set_payment(_: T::Origin, value: T::Balance) -> Result {
		if Self::payment().is_none() {
			<Payment<T>>::put(value);
			<Pot<T>>::put(value);
		}
		Ok(())
	}
}

impl<T: Trait> OnFinalise<T::BlockNumber> for Module<T> {}