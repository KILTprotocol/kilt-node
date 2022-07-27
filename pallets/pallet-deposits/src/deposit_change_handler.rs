use frame_system::pallet_prelude::OriginFor;
use sp_runtime::DispatchResult;

use crate::{Config, DepositIdOf, AccountIdOf, BalanceOf};

pub trait DepositChangeHandler<T: Config> {
    fn on_deposit_paid(
        deposit: &DepositIdOf<T>,
        account: &AccountIdOf<T>,
        amount: BalanceOf<T>,
    );

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

pub trait DepositChecker<T: Config> {
    // check the current total deposit for a given deposit id
    fn check_deposit(deposit_id: &DepositIdOf<T>) -> BalanceOf<T>;

    /// ensure that the deposit is already paid and if not try to get it from the caller
    fn ensure_deposit(origin: OriginFor<T>, deposit_id: DepositIdOf<T>, min_amount: BalanceOf<T>) -> DispatchResult;
}