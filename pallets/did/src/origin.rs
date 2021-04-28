use frame_support::{
	codec::{Decode, Encode},
	traits::EnsureOrigin,
};
use sp_runtime::RuntimeDebug;

/// Origin for the did module.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
pub struct RawOrigin<DidIdentifier> {
	pub id: DidIdentifier,
}

pub struct EnsureDid<DidIdentifier>(sp_std::marker::PhantomData<DidIdentifier>);
impl<O: Into<Result<RawOrigin<DidIdentifier>, O>> + From<RawOrigin<DidIdentifier>>, DidIdentifier> EnsureOrigin<O>
	for EnsureDid<DidIdentifier>
{
	type Success = DidIdentifier;
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| Ok(o.id))
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> O {
		O::from(RawOrigin { id: Default::default() })
	}
}
