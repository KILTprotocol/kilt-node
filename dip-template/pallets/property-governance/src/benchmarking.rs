//! Benchmarking setup for pallet-property-governance
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as PropertyGovernance;
use frame_benchmarking::__private::vec;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::sp_runtime::traits::Bounded;
use pallet_nft_marketplace::Pallet as NftMarketplace;
type DepositBalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
type DepositBalanceOf1<T> = <<T as pallet_property_management::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
type DepositBalanceOf2<T> = <<T as pallet::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
use pallet_xcavate_whitelist::Pallet as Whitelist;
use pallet_property_management::Pallet as PropertyManagement;
use frame_support::{traits::Get, assert_ok};
use frame_support::BoundedVec;
use frame_support::sp_runtime::traits::StaticLookup;
use pallet_assets::Pallet as Assets;

type BalanceOf2<T> = <T as pallet_assets::Config<pallet_assets::Instance1>>::Balance;

fn setup_real_estate_object<T: Config>() -> T::AccountId {
	let value: BalanceOf2<T> = 10000u32.into();
	let caller: T::AccountId = whitelisted_caller();
	let max_balance = DepositBalanceOf::<T>::max_value();
	
	// Ensure the caller has maximum balance
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(&caller, max_balance);

	// Ensure the marketplace accounts have maximum balance
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(&NftMarketplace::<T>::treasury_account_id(), max_balance);
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(&NftMarketplace::<T>::community_account_id(), max_balance);
	
	// Create a new region and location
	assert_ok!(NftMarketplace::<T>::create_new_region(RawOrigin::Root.into()));
	let location: BoundedVec<u8, <T as pallet_nft_marketplace::Config>::PostcodeLimit> = vec![0; <T as pallet_nft_marketplace::Config>::PostcodeLimit::get() as usize]
		.try_into()
		.unwrap();
	assert_ok!(NftMarketplace::<T>::create_new_location(RawOrigin::Root.into(), 0, location.clone()));

	// Add caller to whitelist
	assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
	let user_lookup = <T::Lookup as StaticLookup>::unlookup(caller.clone());
	let asset_id = <T as pallet::Config>::Helper::to_asset(1);
	
	// Create and mint asset
	assert_ok!(Assets::<T, Instance1>::create(RawOrigin::Signed(caller.clone()).into(), asset_id.clone().into(), user_lookup.clone(), 1u32.into()));
	assert_ok!(Assets::<T, Instance1>::mint(RawOrigin::Signed(caller.clone()).into(), asset_id.clone().into(), user_lookup, 1_000_000_000u32.into()));

	// List and buy object
	assert_ok!(NftMarketplace::<T>::list_object(RawOrigin::Signed(caller.clone()).into(), 0, location.clone(), value.into(), 100, vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize].try_into().unwrap()));
	assert_ok!(NftMarketplace::<T>::buy_token(RawOrigin::Signed(caller.clone()).into(), 0, 100));

	// Setup the letting agent with sufficient balance
	let letting_agent: T::AccountId = whitelisted_caller();
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(&letting_agent, max_balance);

	// Ensure the governance and management accounts have maximum balance
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(&PropertyGovernance::<T>::account_id(), max_balance);
	<T as pallet_property_management::Config>::Currency::make_free_balance_be(&letting_agent, DepositBalanceOf1::<T>::max_value());
	<T as pallet_property_management::Config>::Currency::make_free_balance_be(&PropertyManagement::<T>::account_id(), 2_u32.into());
	<T as pallet_property_management::Config>::Currency::make_free_balance_be(&PropertyGovernance::<T>::account_id(), 2_u32.into());
	<T as pallet::Config>::Currency::make_free_balance_be(&PropertyManagement::<T>::account_id(), 1_000_000_000_u32.into());
	<T as pallet::Config>::Currency::make_free_balance_be(&PropertyGovernance::<T>::account_id(), 1_000_000_000_u32.into());
	// Add letting agent and perform necessary operations
	assert_ok!(PropertyManagement::<T>::add_letting_agent(RawOrigin::Root.into(), 0, location.clone(), letting_agent.clone()));
	assert_ok!(PropertyManagement::<T>::letting_agent_deposit(RawOrigin::Signed(letting_agent.clone()).into()));
	assert_ok!(PropertyManagement::<T>::set_letting_agent(RawOrigin::Signed(letting_agent.clone()).into(), 0));	

	// Distribute income
	<T as pallet_property_management::Config>::Currency::make_free_balance_be(&letting_agent, DepositBalanceOf1::<T>::max_value());
	assert_ok!(PropertyManagement::<T>::distribute_income(RawOrigin::Signed(letting_agent.clone()).into(), 0, 5000_u32.into()));
	<T as pallet_property_management::Config>::Currency::make_free_balance_be(&PropertyGovernance::<T>::account_id(), DepositBalanceOf1::<T>::max_value());

	letting_agent
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn propose() {
		let letting_agent = setup_real_estate_object::<T>();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&letting_agent,
			DepositBalanceOf::<T>::max_value(),
		);
		assert!(PropertyManagement::<T>::property_reserve(0) > 1_u32.into());
		<T as pallet::Config>::Currency::make_free_balance_be(
			&PropertyGovernance::<T>::account_id(),
			DepositBalanceOf2::<T>::max_value(),
		); 
		<T as pallet::Config>::Currency::make_free_balance_be(
			&letting_agent,
			1_000_000_000_u32.into(),
		); 
		#[extrinsic_call]
		propose(RawOrigin::Signed(letting_agent.clone()), 0, 1_u32.into(), vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
			.try_into()
			.unwrap());

		assert_eq!(PropertyGovernance::<T>::proposals(1).is_some(), false);
	}

  	#[benchmark]
	fn challenge_against_letting_agent() {
		let _ = setup_real_estate_object::<T>();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&caller,
			DepositBalanceOf::<T>::max_value(),
		);
		#[extrinsic_call]
		challenge_against_letting_agent(RawOrigin::Signed(caller.clone()), 0);

		assert_eq!(PropertyGovernance::<T>::challenges(1).is_some(), true);
	}

	#[benchmark]
	fn vote_on_proposal() {
		let letting_agent = setup_real_estate_object::<T>();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&caller,
			DepositBalanceOf::<T>::max_value(),
		);
		<T as pallet::Config>::Currency::make_free_balance_be(
			&PropertyGovernance::<T>::account_id(),
			DepositBalanceOf2::<T>::max_value(),
		); 
		<T as pallet::Config>::Currency::make_free_balance_be(
			&letting_agent,
			1_000_000_000_u32.into(),
		); 
		assert_ok!{PropertyGovernance::<T>::propose(RawOrigin::Signed(letting_agent.clone()).into(), 0, 1500_u32.into(), vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
			.try_into()
			.unwrap())};
		#[extrinsic_call]
		vote_on_proposal(RawOrigin::Signed(caller.clone()), 1, crate::Vote::Yes);

		assert_eq!(PropertyGovernance::<T>::ongoing_votes(1).unwrap().yes_votes, 100);
		assert_eq!(PropertyGovernance::<T>::proposal_voter(1).len(), 1);
	}

	#[benchmark]
	fn vote_on_letting_agent_challenge() {
		let _ = setup_real_estate_object::<T>();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&caller,
			DepositBalanceOf::<T>::max_value(),
		);
		assert_ok!{PropertyGovernance::<T>::challenge_against_letting_agent(RawOrigin::Signed(caller.clone()).into(), 0)};
		#[extrinsic_call]
		vote_on_letting_agent_challenge(RawOrigin::Signed(caller.clone()), 1, crate::Vote::Yes);

		assert_eq!(PropertyGovernance::<T>::ongoing_challenge_votes(1, crate::ChallengeState::First).unwrap().yes_votes, 100);
		assert_eq!(PropertyGovernance::<T>::challenge_voter(1, crate::ChallengeState::First).len(), 1);
	}  


	impl_benchmark_test_suite!(PropertyGovernance, crate::mock::new_test_ext(), crate::mock::Test);
}
