use frame_support::traits::Contains;

use super::Call;

pub struct GovCalls;
impl Contains<Call> for GovCalls {
	fn contains(t: &Call) -> bool {
		// We don't want to disable governance completely since we need it to set the
		// filter.
		matches!(t, Call::Treasury(..) | Call::TipsMembership(..) | Call::Tips(..))
	}
}

pub struct StakeCalls;
impl Contains<Call> for StakeCalls {
	fn contains(t: &Call) -> bool {
		matches!(t, Call::ParachainStaking(..) | Call::Session(..))
	}
}

pub struct TransferCalls;
impl Contains<Call> for TransferCalls {
	fn contains(t: &Call) -> bool {
		matches!(
			t,
			Call::Balances(..)
				| Call::Indices(pallet_indices::Call::force_transfer { .. } | pallet_indices::Call::transfer { .. })
				| Call::Vesting(
					pallet_vesting::Call::force_vested_transfer { .. } | pallet_vesting::Call::vested_transfer { .. }
				)
		)
	}
}

pub struct FeatureCalls;
impl Contains<Call> for FeatureCalls {
	fn contains(t: &Call) -> bool {
		matches!(
			t,
			Call::Attestation(..)
				| Call::Ctype(..)
				| Call::Delegation(..)
				| Call::Did(..) | Call::DidLookup(..)
				| Call::Web3Names(..)
		)
	}
}

pub struct XcmCalls;
impl Contains<Call> for XcmCalls {
	fn contains(t: &Call) -> bool {
		matches!(t, Call::PolkadotXcm(..))
	}
}

pub struct SystemCalls;
impl Contains<Call> for SystemCalls {
	fn contains(t: &Call) -> bool {
		matches!(
			t,
			Call::System(_) | Call::ParachainSystem(..) | Call::Timestamp(..) | Call::Sudo(..)
		)
	}
}
