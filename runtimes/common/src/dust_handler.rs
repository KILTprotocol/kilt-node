use core::marker::PhantomData;
use frame_support::traits::{fungible::Credit, OnUnbalanced};
use sp_runtime::SaturatedConversion;
pub struct DustHandler<T>(PhantomData<T>);

type CreditOf<T> = Credit<<T as frame_system::Config>::AccountId, <T as pallet_balances::Config>::Balance>;

impl<T> OnUnbalanced<CreditOf<T>> for DustHandler<T>
where
	T: pallet_balances::Config,
	<T as pallet_balances::Config>::Balance:
		frame_support::traits::fungible::Balanced<<T as frame_system::Config>::AccountId> + From<u128>,
{
	fn on_nonzero_unbalanced(amount: CreditOf<T>) {
		let p: <T as pallet_balances::Config>::Balance = amount.saturated_into();
	}
}
