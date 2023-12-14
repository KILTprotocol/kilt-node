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

//! Collection of support traits, types, and functions for integrating KILT as
//! an identity provider following the Decentralized Identity Provider (DIP)
//! protocol.
//!
//! Consumers of KILT identities should prefer directly using
//! [`KiltVersionedRelaychainVerifier`] for consumer relaychains and
//! [`KiltVersionedParachainVerifier`] for consumer sibling parachains.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod did;
pub mod merkle;
pub mod state_proofs;
pub mod traits;
pub mod utils;
pub mod verifier;

pub use state_proofs::relaychain::RelayStateRootsViaRelayStorePallet;
pub use traits::{FrameSystemDidSignatureContext, ProviderParachainStateInfoViaProviderPallet};
pub use utils::BoundedBlindedValue;
pub use verifier::*;
