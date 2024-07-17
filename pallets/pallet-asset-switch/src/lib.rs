// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

pub mod traits;
pub mod xcm;

mod default_weights;
pub use default_weights::WeightInfo;
mod switch;
pub use switch::{SwitchPairInfo, SwitchPairStatus};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(feature = "runtime-benchmarks")]
pub use benchmarking::{BenchmarkHelper, BenchmarkInfo};

use ::xcm::{VersionedAssetId, VersionedMultiAsset, VersionedMultiLocation};
use frame_support::traits::PalletInfoAccess;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::TrailingZeroInput;
use sp_std::boxed::Box;

pub use crate::pallet::*;

const LOG_TARGET: &str = "runtime::pallet-asset-switch";

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::{
		switch::{SwitchPairInfo, SwitchPairStatus},
		traits::SwitchHooks,
		WeightInfo, LOG_TARGET,
	};

	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{Inspect as InspectFungible, Mutate as MutateFungible},
			tokens::{Fortitude, Preservation, Provenance},
			EnsureOrigin,
		},
	};
	use frame_system::{ensure_root, pallet_prelude::*};
	use sp_runtime::traits::TryConvert;
	use sp_std::{boxed::Box, vec};
	use xcm::{
		v3::{
			validate_send, AssetId,
			Instruction::{BuyExecution, DepositAsset, RefundSurplus, SetAppendix, TransferAsset, WithdrawAsset},
			Junction, MultiAsset, MultiAssetFilter, MultiLocation, SendXcm, WeightLimit, WildMultiAsset, Xcm,
		},
		VersionedAssetId, VersionedMultiAsset, VersionedMultiLocation,
	};
	use xcm_executor::traits::TransactAsset;

	pub type LocalCurrencyBalanceOf<T, I> =
		<<T as Config<I>>::LocalCurrency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;
	pub type SwitchPairInfoOf<T> = SwitchPairInfo<<T as frame_system::Config>::AccountId>;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// How to convert a local `AccountId` to a `Junction`, for the purpose
		/// of taking XCM fees from the user's balance via the configured
		/// `AssetTransactor`.
		type AccountIdConverter: TryConvert<Self::AccountId, Junction>;
		/// The asset transactor to charge user's for XCM fees as specified in
		/// the switch pair.
		type AssetTransactor: TransactAsset;
		/// The origin that can update the XCM fee for a switch pair.
		type FeeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The local currency.
		type LocalCurrency: MutateFungible<Self::AccountId>;
		/// The origin that can pause switches in both directions.
		type PauseOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The aggregate event type.
		type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The origin that can request a switch of some local tokens for some
		/// remote assets.
		type SubmitterOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// Runtime-injected logic to execute before and after a local -> remote
		/// and remote -> local switch.
		type SwitchHooks: SwitchHooks<Self, I>;
		/// The origin that can set a new switch pair, remove one, or resume
		/// switches.
		type SwitchOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type WeightInfo: WeightInfo;
		/// The XCM router to route XCM transfers to the configured reserve
		/// location.
		type XcmRouter: SendXcm;

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: crate::benchmarking::BenchmarkHelper;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A new switch pair is created.
		SwitchPairCreated {
			circulating_supply: u128,
			pool_account: T::AccountId,
			remote_asset_id: VersionedAssetId,
			remote_asset_reserve_location: VersionedMultiLocation,
			remote_xcm_fee: Box<VersionedMultiAsset>,
			total_issuance: u128,
		},
		/// A switch pair is removed.
		SwitchPairRemoved { remote_asset_id: VersionedAssetId },
		/// A switch pair has enabled switches.
		SwitchPairResumed { remote_asset_id: VersionedAssetId },
		/// A switch pair has suspended switches.
		SwitchPairPaused { remote_asset_id: VersionedAssetId },
		/// The XCM fee for the switch has been updated.
		SwitchPairFeeUpdated {
			old: VersionedMultiAsset,
			new: VersionedMultiAsset,
		},
		/// A switch of local -> remote asset has taken place.
		LocalToRemoteSwitchExecuted {
			from: T::AccountId,
			to: VersionedMultiLocation,
			amount: LocalCurrencyBalanceOf<T, I>,
		},
		/// A switch of remote -> local asset has taken place.
		RemoteToLocalSwitchExecuted { to: T::AccountId, amount: u128 },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Provided switch pair info is not valid.
		InvalidInput,
		/// The runtime-injected logic returned an error with a specific code.
		Hook(u8),
		/// There are not enough remote assets to cover the specified amount of
		/// local tokens to switch.
		Liquidity,
		/// Failure in transferring the local tokens from the user's balance to
		/// the switch pair pool account.
		LocalPoolBalance,
		/// The calculated switch pair pool account does not have enough local
		/// tokens to cover the specified `circulating_supply`.
		PoolInitialLiquidityRequirement,
		/// A switch pair has already been set.
		SwitchPairAlreadyExisting,
		/// The switch pair did not enable switches.
		SwitchPairNotEnabled,
		/// No switch pair found.
		SwitchPairNotFound,
		/// The user does not have enough local tokens to cover the requested
		/// switch.
		UserSwitchBalance,
		/// The user does not have enough assets to pay for the remote XCM fees.
		UserXcmBalance,
		/// Something regarding XCM went wrong.
		Xcm,
		/// Internal error.
		Internal,
	}

	#[pallet::storage]
	#[pallet::getter(fn switch_pair)]
	pub(crate) type SwitchPair<T: Config<I>, I: 'static = ()> = StorageValue<_, SwitchPairInfoOf<T>, OptionQuery>;

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I>
	where
		LocalCurrencyBalanceOf<T, I>: Into<u128>,
	{
		/// Set a new switch pair.
		///
		/// See the crate's README for more.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config<I>>::WeightInfo::set_switch_pair())]
		pub fn set_switch_pair(
			origin: OriginFor<T>,
			reserve_location: Box<VersionedMultiLocation>,
			remote_asset_id: Box<VersionedAssetId>,
			remote_fee: Box<VersionedMultiAsset>,
			total_issuance: u128,
			circulating_supply: u128,
		) -> DispatchResult {
			T::SwitchOrigin::ensure_origin(origin)?;

			// 1. Verify switch pair has not already been set.
			ensure!(!SwitchPair::<T, I>::exists(), Error::<T, I>::SwitchPairAlreadyExisting);

			// 2. Verify that total issuance >= circulating supply.
			ensure!(total_issuance >= circulating_supply, Error::<T, I>::InvalidInput);

			// 3. Verify the pool account has enough local assets to match the circulating
			//    supply of eKILTs to cover for all potential remote -> local switches.
			let pool_account = Self::pool_account_id_for_remote_asset(&remote_asset_id)?;
			let pool_account_reducible_balance_as_u128: u128 =
				T::LocalCurrency::reducible_balance(&pool_account, Preservation::Expendable, Fortitude::Polite).into();
			ensure!(
				pool_account_reducible_balance_as_u128 >= circulating_supply,
				Error::<T, I>::PoolInitialLiquidityRequirement
			);

			Self::set_switch_pair_bypass_checks(
				*reserve_location,
				*remote_asset_id,
				*remote_fee,
				total_issuance,
				circulating_supply,
				pool_account,
			);

			Ok(())
		}

		/// Force-set a new switch pair.
		///
		/// See the crate's README for more.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config<I>>::WeightInfo::force_set_switch_pair())]
		pub fn force_set_switch_pair(
			origin: OriginFor<T>,
			reserve_location: Box<VersionedMultiLocation>,
			remote_asset_id: Box<VersionedAssetId>,
			remote_fee: Box<VersionedMultiAsset>,
			total_issuance: u128,
			circulating_supply: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(total_issuance >= circulating_supply, Error::<T, I>::InvalidInput);
			let pool_account = Self::pool_account_id_for_remote_asset(&remote_asset_id)?;

			Self::set_switch_pair_bypass_checks(
				*reserve_location,
				*remote_asset_id,
				*remote_fee,
				total_issuance,
				circulating_supply,
				pool_account,
			);

			Ok(())
		}

		/// Unset a switch pair.
		///
		/// See the crate's README for more.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config<I>>::WeightInfo::force_unset_switch_pair())]
		pub fn force_unset_switch_pair(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;

			Self::unset_switch_pair_bypass_checks();

			Ok(())
		}

		/// Pause switches for a switch pair.
		///
		/// See the crate's README for more.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config<I>>::WeightInfo::pause_switch_pair())]
		pub fn pause_switch_pair(origin: OriginFor<T>) -> DispatchResult {
			T::PauseOrigin::ensure_origin(origin)?;

			Self::set_switch_pair_status(SwitchPairStatus::Paused)?;

			Ok(())
		}

		/// Resume switches for a switch pair.
		///
		/// See the crate's README for more.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config<I>>::WeightInfo::resume_switch_pair())]
		pub fn resume_switch_pair(origin: OriginFor<T>) -> DispatchResult {
			T::SwitchOrigin::ensure_origin(origin)?;

			Self::set_switch_pair_status(SwitchPairStatus::Running)?;

			Ok(())
		}

		/// Update the remote XCM fee for a switch pair.
		///
		/// See the crate's README for more.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config<I>>::WeightInfo::update_remote_fee())]
		pub fn update_remote_fee(origin: OriginFor<T>, new: Box<VersionedMultiAsset>) -> DispatchResult {
			T::FeeOrigin::ensure_origin(origin)?;

			SwitchPair::<T, I>::try_mutate(|entry| {
				let SwitchPairInfoOf::<T> { remote_fee, .. } =
					entry.as_mut().ok_or(Error::<T, I>::SwitchPairNotFound)?;
				let old_remote_fee = remote_fee.clone();
				*remote_fee = *new.clone();
				if old_remote_fee != *new {
					Self::deposit_event(Event::<T, I>::SwitchPairFeeUpdated {
						old: old_remote_fee,
						new: *new,
					});
				};
				Ok::<_, Error<T, I>>(())
			})?;

			Ok(())
		}

		/// Perform a local -> remote asset switch.
		///
		/// See the crate's README for more.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config<I>>::WeightInfo::switch())]
		pub fn switch(
			origin: OriginFor<T>,
			local_asset_amount: LocalCurrencyBalanceOf<T, I>,
			beneficiary: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			let submitter = T::SubmitterOrigin::ensure_origin(origin)?;

			// 1. Retrieve switch pair info from storage, else fail.
			let switch_pair =
				SwitchPair::<T, I>::get().ok_or(DispatchError::from(Error::<T, I>::SwitchPairNotFound))?;

			// 2. Check if switches are enabled.
			ensure!(
				switch_pair.can_switch(),
				DispatchError::from(Error::<T, I>::SwitchPairNotEnabled)
			);

			// 3. Verify the tx submitter has enough local assets for the switch, without
			//    having their balance go to zero.
			T::LocalCurrency::can_withdraw(&submitter, local_asset_amount)
				.into_result(true)
				.map_err(|_| DispatchError::from(Error::<T, I>::UserSwitchBalance))?;

			// 4. Verify the local assets can be transferred to the switch pool account
			T::LocalCurrency::can_deposit(&switch_pair.pool_account, local_asset_amount, Provenance::Extant)
				.into_result()
				.map_err(|_| DispatchError::from(Error::<T, I>::LocalPoolBalance))?;

			// 5. Verify we have enough balance on the remote location to perform the
			//    transfer
			let remote_asset_amount_as_u128 = local_asset_amount.into();
			ensure!(
				switch_pair.remote_asset_balance >= remote_asset_amount_as_u128,
				Error::<T, I>::Liquidity
			);

			let asset_id_v3: AssetId = switch_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert asset ID {:?} into v3 `AssetId` with error {:?}",
					switch_pair.remote_asset_id,
					e
				);
				DispatchError::from(Error::<T, I>::Internal)
			})?;
			let remote_asset_fee_v3: MultiAsset = switch_pair.remote_fee.clone().try_into().map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert remote XCM asset fee {:?} into v3 `MultiAssset` with error {:?}",
					switch_pair.remote_fee,
					e
				);
				DispatchError::from(Error::<T, I>::Xcm)
			})?;
			let destination_v3: MultiLocation =
				switch_pair.remote_reserve_location.clone().try_into().map_err(|e| {
					log::error!(
						target: LOG_TARGET,
						"Failed to convert remote reserve location {:?} into v3 `MultiLocation` with error {:?}",
						switch_pair.remote_reserve_location,
						e
					);
					DispatchError::from(Error::<T, I>::Internal)
				})?;
			let beneficiary_v3: MultiLocation = (*beneficiary.clone()).try_into().map_err(|e| {
				log::info!(
					target: LOG_TARGET,
					"Failed to convert beneficiary {:?} into v3 `MultiLocation` with error {:?}",
					beneficiary,
					e
				);
				DispatchError::from(Error::<T, I>::Xcm)
			})?;
			// Use the same local `AccountIdConverter` to generate a `MultiLocation` to use
			// to send funds on remote.
			let submitter_as_multilocation = T::AccountIdConverter::try_convert(submitter.clone())
				.map(|j| j.into_location())
				.map_err(|e| {
					log::info!(
						target: LOG_TARGET,
						"Failed to convert account {:?} into `MultiLocation` with error {:?}",
						submitter,
						e
					);
					DispatchError::from(Error::<T, I>::Xcm)
				})?;

			// 6. Compose and validate XCM message
			let appendix: Xcm<()> = vec![
				RefundSurplus,
				DepositAsset {
					assets: MultiAssetFilter::Wild(WildMultiAsset::All),
					beneficiary: submitter_as_multilocation,
				},
			]
			.into();
			let remote_xcm: Xcm<()> = vec![
				WithdrawAsset(remote_asset_fee_v3.clone().into()),
				BuyExecution {
					weight_limit: WeightLimit::Unlimited,
					fees: remote_asset_fee_v3.clone(),
				},
				TransferAsset {
					assets: (asset_id_v3, remote_asset_amount_as_u128).into(),
					beneficiary: beneficiary_v3,
				},
				SetAppendix(appendix),
			]
			.into();
			let xcm_ticket = validate_send::<T::XcmRouter>(destination_v3, remote_xcm.clone()).map_err(|e| {
				log::info!(
					"Failed to call validate_send for destination {:?} and remote XCM {:?} with error {:?}",
					destination_v3,
					remote_xcm,
					e
				);
				DispatchError::from(Error::<T, I>::Xcm)
			})?;

			// 7. Call into hook pre-switch checks
			T::SwitchHooks::pre_local_to_remote_switch(&submitter, &beneficiary, local_asset_amount)
				.map_err(|e| DispatchError::from(Error::<T, I>::Hook(e.into())))?;

			// 8. Transfer funds from user to pool
			let transferred_amount = T::LocalCurrency::transfer(
				&submitter,
				&switch_pair.pool_account,
				local_asset_amount,
				Preservation::Preserve,
			)?;
			if transferred_amount != local_asset_amount {
				log::error!(
					"Transferred amount {:?} does not match expected user-specified amount {:?}",
					transferred_amount,
					local_asset_amount
				);
				return Err(Error::<T, I>::Internal.into());
			}

			// 9. Take XCM fee from submitter.
			let withdrawn_fees =
				T::AssetTransactor::withdraw_asset(&remote_asset_fee_v3, &submitter_as_multilocation, None).map_err(
					|e| {
						log::info!(
							target: LOG_TARGET,
							"Failed to withdraw asset {:?} from location {:?} with error {:?}",
							remote_asset_fee_v3,
							submitter_as_multilocation,
							e
						);
						DispatchError::from(Error::<T, I>::UserXcmBalance)
					},
				)?;
			if withdrawn_fees != vec![remote_asset_fee_v3.clone()].into() {
				log::error!(
					target: LOG_TARGET,
					"Withdrawn fees {:?} does not match expected fee {:?}.",
					withdrawn_fees,
					remote_asset_fee_v3
				);
				return Err(DispatchError::from(Error::<T, I>::Internal));
			}

			// 10. Send XCM out (only when not benchmarking, as delivery fees are anyway
			//     accounted for by the router)
			cfg_if::cfg_if! {
				if #[cfg(not(feature = "runtime-benchmarks"))] {
					T::XcmRouter::deliver(xcm_ticket.0).map_err(|e| {
						log::info!("Failed to deliver ticket with error {:?}", e);
						DispatchError::from(Error::<T, I>::Xcm)
					})?;
				} else {
					log::trace!(target: LOG_TARGET, "Running benchmarks. Message will not be delivered to destination.");
					drop(xcm_ticket);
				}
			}

			// 11. Update remote asset balance
			SwitchPair::<T, I>::try_mutate(|entry| {
				let Some(SwitchPairInfoOf::<T> {
					remote_asset_balance, ..
				}) = entry.as_mut()
				else {
					log::error!(target: LOG_TARGET, "Failed to borrow stored switch pair info as mut.");
					return Err(Error::<T, I>::Internal);
				};
				let Some(new_balance) = remote_asset_balance.checked_sub(remote_asset_amount_as_u128) else {
					log::error!(target: LOG_TARGET, "Failed to subtract {:?} from stored remote balance {:?}.", transferred_amount, remote_asset_balance);
					return Err(Error::<T, I>::Internal);
				};
				*remote_asset_balance = new_balance;
				Ok(())
			})?;

			// 12. Call into hook post-switch checks
			T::SwitchHooks::post_local_to_remote_switch(&submitter, &beneficiary, local_asset_amount)
				.map_err(|e| DispatchError::from(Error::<T, I>::Hook(e.into())))?;

			Self::deposit_event(Event::<T, I>::LocalToRemoteSwitchExecuted {
				from: submitter,
				to: *beneficiary,
				amount: local_asset_amount,
			});

			Ok(())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	fn set_switch_pair_bypass_checks(
		reserve_location: VersionedMultiLocation,
		remote_asset_id: VersionedAssetId,
		remote_fee: VersionedMultiAsset,
		total_issuance: u128,
		circulating_supply: u128,
		pool_account: T::AccountId,
	) {
		debug_assert!(
			total_issuance >= circulating_supply,
			"Provided total issuance smaller than circulating supply."
		);
		let switch_pair_info = SwitchPairInfoOf::<T> {
			pool_account: pool_account.clone(),
			// We can do a simple subtraction since all checks are performed in calling functions.
			remote_asset_balance: total_issuance - circulating_supply,
			remote_asset_id: remote_asset_id.clone(),
			remote_fee: remote_fee.clone(),
			remote_reserve_location: reserve_location.clone(),
			status: SwitchPairStatus::Paused,
		};

		SwitchPair::<T, I>::set(Some(switch_pair_info));

		Self::deposit_event(Event::<T, I>::SwitchPairCreated {
			circulating_supply,
			pool_account,
			remote_asset_reserve_location: reserve_location,
			remote_asset_id,
			remote_xcm_fee: Box::new(remote_fee),
			total_issuance,
		});
	}

	fn unset_switch_pair_bypass_checks() {
		let switch_pair = SwitchPair::<T, I>::take();
		if let Some(switch_pair) = switch_pair {
			Self::deposit_event(Event::<T, I>::SwitchPairRemoved {
				remote_asset_id: switch_pair.remote_asset_id,
			});
		};
	}

	fn set_switch_pair_status(new_status: SwitchPairStatus) -> Result<(), Error<T, I>> {
		SwitchPair::<T, I>::try_mutate(|entry| {
			let SwitchPairInfoOf::<T> {
				remote_asset_id,
				status,
				..
			} = entry.as_mut().ok_or(Error::<T, I>::SwitchPairNotFound)?;
			let relevant_event = match new_status {
				SwitchPairStatus::Running => Event::<T, I>::SwitchPairResumed {
					remote_asset_id: remote_asset_id.clone(),
				},
				SwitchPairStatus::Paused => Event::<T, I>::SwitchPairPaused {
					remote_asset_id: remote_asset_id.clone(),
				},
			};
			let old_status = status.clone();
			*status = new_status;
			// If state was actually changed, generate an event, otherwise this is a no-op.
			if old_status != *status {
				Self::deposit_event(relevant_event);
			}
			Ok::<_, Error<T, I>>(())
		})?;
		Ok(())
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Derive an `AccountId` for the provided `remote_asset_id` and the
	/// pallet's name as configured in the runtime.
	pub fn pool_account_id_for_remote_asset(remote_asset_id: &VersionedAssetId) -> Result<T::AccountId, Error<T, I>> {
		let pallet_name = <Pallet<T, I> as PalletInfoAccess>::name();
		let pallet_name_hashed = sp_io::hashing::blake2_256(pallet_name.as_bytes());
		let hash_input = (pallet_name_hashed, b'.', remote_asset_id.clone()).encode();
		let hash_output = sp_io::hashing::blake2_256(hash_input.as_slice());
		T::AccountId::decode(&mut TrailingZeroInput::new(hash_output.as_slice())).map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to generate pool ID from remote asset {:?} with error: {:?}",
				remote_asset_id,
				e
			);
			Error::<T, I>::Internal
		})
	}
}
