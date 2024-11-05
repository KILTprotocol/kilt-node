use frame_benchmarking::v2::*;
use sp_core::Get;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::traits::{Fixed, ToFixed};

use crate::{
	curves::CurveInput,
	mock::{get_linear_bonding_curve_input, DEFAULT_COLLATERAL_CURRENCY_ID},
	Call, Config, CurveParameterTypeOf, Pallet, TokenMetaOf,
};

#[benchmarks(where <CurveParameterTypeOf<T> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign)]
mod benchmarks {

	use frame_system::RawOrigin;
	use sp_runtime::BoundedVec;

	use crate::{curves::polynomial::PolynomialParametersInput, CurveParameterInputOf};

	use super::*;

	// fn get_max_currencies<T: Config>() -> Vec<TokenMetaOf<T>> {
	// 	let max_currencies = T::MaxCurrencies::get();
	// 	vec![TokenMetaOf { ..Default::default() }; max_currencies as usize]
	// }

	#[benchmark]
	fn create_pool() {
		let caller = whitelisted_caller();

		let a = CurveParameterInputOf::<T>::from_num(0);

		let curve = CurveInput::Polynomial(PolynomialParametersInput { m: a, n: a, o: a });
		let collateral_id = DEFAULT_COLLATERAL_CURRENCY_ID;

		let currencies = BoundedVec::try_from(vec![]).expect("Failed to create BoundedVec");
		let origin = RawOrigin::Signed(caller).into();

		#[extrinsic_call]
		Pallet::<T>::create_pool(origin, curve, collateral_id, currencies, 10, true);
	}
}
