use frame_support::traits::Contains;

use super::Call;

pub struct TransferCalls;
impl Contains<Call> for TransferCalls {
	fn contains(t: &Call) -> bool {
		matches!(
			t,
			Call::Balances(..)
				| Call::Indices(pallet_indices::Call::force_transfer { .. } | pallet_indices::Call::transfer { .. })
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
	fn contains(_: &Call) -> bool {
		false
	}
}

pub struct SystemCalls;
impl Contains<Call> for SystemCalls {
	fn contains(t: &Call) -> bool {
		matches!(t, Call::System(..) | Call::Sudo(..) | Call::Timestamp(..))
	}
}
