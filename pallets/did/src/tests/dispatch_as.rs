// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use frame_support::{assert_noop, assert_ok};

use crate::{
	did_details::DidDetails,
	mock::{Did, ExtBuilder, RuntimeCall, RuntimeOrigin, Test, DEFAULT_BALANCE},
	AccountIdOf, Did as DidStorage, DidIdentifierOf, Error,
};

mod attestation;
mod authentication;
mod delegation;
mod error_cases;

fn blueprint_successful_dispatch<FB: FnOnce() -> (), FA: FnOnce() -> ()>(
	did_identifier: DidIdentifierOf<Test>,
	caller: AccountIdOf<Test>,
	did_details: DidDetails<Test>,
	call: RuntimeCall,
	before: FB,
	after: FA,
) {
	ExtBuilder::default()
		.with_balances(vec![(did_details.deposit.owner.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did_identifier.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			before();
			let did_details_before = DidStorage::<Test>::get(did_identifier.clone()).expect("Did must exists");

			assert_ok!(Did::dispatch_as(
				RuntimeOrigin::signed(caller),
				did_identifier.clone(),
				Box::new(call),
			));

			let did_details_after = DidStorage::<Test>::get(did_identifier.clone()).expect("Did must not be deleted");
			assert_eq!(did_details_before, did_details_after, "Did details must not be changed");

			after();
		});
}

fn blueprint_failed_dispatch<F: FnOnce() -> ()>(
	did_identifier: DidIdentifierOf<Test>,
	caller: AccountIdOf<Test>,
	did_details: Option<DidDetails<Test>>,
	call: RuntimeCall,
	before: F,
	error: Error<Test>,
) {
	let (balances, dids) = if let Some(did_details) = did_details {
		(
			vec![(did_details.deposit.owner.clone(), DEFAULT_BALANCE)],
			vec![(did_identifier.clone(), did_details)],
		)
	} else {
		(Vec::new(), Vec::new())
	};
	ExtBuilder::default()
		.with_balances(balances)
		.with_dids(dids)
		.build_and_execute_with_sanity_tests(None, || {
			before();
			assert_noop!(
				Did::dispatch_as(RuntimeOrigin::signed(caller), did_identifier.clone(), Box::new(call),),
				error
			);
		});
}
