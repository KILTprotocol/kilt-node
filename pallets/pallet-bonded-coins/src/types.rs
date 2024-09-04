use frame_support::BoundedVec;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::Get;

#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum EventState<CurrencyId> {
	#[default]
	Active,
	Frozen,
	Sealed, /// TODO: save block number where seal happened here
	Decided(CurrencyId),
	Destroying,
}

#[derive(Default, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct EventData<AccountId, CurrencyId, MaxNameLength: Get<u32>, MaxOptions: Get<u32>> {
	pub creator: AccountId,
	pub name: BoundedVec<u8, MaxNameLength>,
	pub linked_currencies: BoundedVec<CurrencyId, MaxOptions>,
	pub state: EventState<CurrencyId>,
}

impl<AccountId, CurrencyId, MaxNameLength: Get<u32>, MaxOptions: Get<u32>>
	EventData<AccountId, CurrencyId, MaxNameLength, MaxOptions>
{
	pub fn new(
		creator: AccountId,
		name: BoundedVec<u8, MaxNameLength>,
		linked_currencies: BoundedVec<CurrencyId, MaxOptions>,
	) -> Self {
		Self {
			creator,
			name,
			linked_currencies,
			state: EventState::default(),
		}
	}
}
