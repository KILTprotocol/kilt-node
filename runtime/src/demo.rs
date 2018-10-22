/*extern crate sr_std;
#[cfg(feature = "std")] #[macro_use] extern crate serde_derive;	//< ???
#[macro_use] extern crate parity_codec_derive; //< ???
extern crate parity_codec as codec;	//< ???

#[macro_use] extern crate srml_support as support;
extern crate srml_system as system;
extern crate srml_balances as balances;
*/
use parity_codec::Encode;
use srml_support::{StorageValue, dispatch::Result};
use runtime_primitives::traits::{As, Hash, OnFinalise};
use {balances, system::{self, ensure_signed}};

pub trait Trait: balances::Trait {}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn play(origin) -> Result;
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Demo {
		Payment get(payment) config(): T::Balance = T::Balance::sa(1000000);
		Pot get(pot): T::Balance = T::Balance::sa(1000000);
	}
}

impl<T: Trait> Module<T> {
	fn play(origin: T::Origin) -> Result {
		let sender = ensure_signed(origin)?;
		let payment = Self::payment();

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
}

impl<T: Trait> OnFinalise<T::BlockNumber> for Module<T> {}