use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// Creation details of a CType.
#[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, Eq, PartialEq, TypeInfo)]
pub struct CtypeEntry<Creator, BlockNumber> {
	/// Identifier of the creator.
	pub creator: Creator,
	/// Block number in which the creation tx was dispatched.
	pub created_at: BlockNumber,
}
