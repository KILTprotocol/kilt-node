use frame_support::traits::Contains;

use super::RuntimeCall;

pub struct TransferRuntimeCalls;
impl Contains<RuntimeCall> for TransferRuntimeCalls {
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			RuntimeCall::Balances(..)
				| RuntimeCall::Indices(pallet_indices::RuntimeCall::force_transfer { .. } | pallet_indices::RuntimeCall::transfer { .. })
		)
	}
}

pub struct FeatureRuntimeCalls;
impl Contains<RuntimeCall> for FeatureRuntimeCalls {
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			RuntimeCall::Attestation(..)
				| RuntimeCall::Ctype(..)
				| RuntimeCall::Delegation(..)
				| RuntimeCall::Did(..) | RuntimeCall::DidLookup(..)
				| RuntimeCall::Web3Names(..)
		)
	}
}

pub struct XcmRuntimeCalls;
impl Contains<RuntimeCall> for XcmRuntimeCalls {
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

pub struct SystemRuntimeCalls;
impl Contains<RuntimeCall> for SystemRuntimeCalls {
	fn contains(t: &RuntimeCall) -> bool {
		matches!(t, RuntimeCall::System(..) | RuntimeCall::Sudo(..) | RuntimeCall::Timestamp(..))
	}
}
