use frame_system::pallet_prelude::OriginFor;
use sp_runtime::DispatchResult;

use crate::{Config, DepositIdOf, AccountIdOf, BalanceOf};

/// DepositChangeHandler is meant to be implemented on the runtime level to allow it to react to the event when a deposit changed
pub trait DepositChangeHandler<T: Config> {
    /// on_deposit_paid is called when a deposit was paid.
    fn on_deposit_paid(
        deposit: &DepositIdOf<T>,
        account: &AccountIdOf<T>,
        amount: BalanceOf<T>,
    );

    /// on_deposit_take_announcement is called when a deposit was withdrawn.
    fn on_deposit_taken(
        deposit: &DepositIdOf<T>,
        account: &AccountIdOf<T>,
        amount: BalanceOf<T>,
    );
}

impl<T: Config> DepositChangeHandler<T> for () {
    fn on_deposit_paid(_deposit: &DepositIdOf<T>, _account: &AccountIdOf<T>, _amount: BalanceOf<T>) {
        // noop
    }
    fn on_deposit_taken(_deposit: &DepositIdOf<T>, _account: &AccountIdOf<T>, _amount: BalanceOf<T>) {
        // noop
    }
}

// DepositChecker is implemented by the pallet and is meant to be used in other pallets which need deposits to be checked.
pub trait DepositChecker<T: Config> {
    // check the current total deposit for a given deposit id
    fn check_deposit(deposit_id: &DepositIdOf<T>) -> BalanceOf<T>;

    /// ensure that the deposit is already paid and if not try to get it from the caller
    fn ensure_deposit(origin: OriginFor<T>, deposit_id: DepositIdOf<T>, min_amount: BalanceOf<T>) -> DispatchResult;
}