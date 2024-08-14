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

pub mod convert;
pub use convert::AccountId32ToAccountId32JunctionConverter;

pub mod r#match;
pub use r#match::MatchesSwitchPairXcmFeeFungibleAsset;

pub mod transfer;
pub use transfer::{IsSwitchPairRemoteAsset, IsSwitchPairXcmFeeAsset};

pub mod trade;
pub use trade::{UsingComponentsForSwitchPairRemoteAsset, UsingComponentsForXcmFeeAsset};

pub mod transact;
pub use transact::SwitchPairRemoteAssetTransactor;

#[cfg(test)]
mod test_utils;
