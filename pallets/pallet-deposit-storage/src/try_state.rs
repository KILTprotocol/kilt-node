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

//! Pallet to store namespaced deposits for the configured `Currency`. It allows
//! the original payer of a deposit to claim it back, triggering a hook to
//! optionally perform related actions somewhere else in the runtime.
//! Each deposit is identified by a namespace and a key. There cannot be two
//! equal keys under the same namespace, but the same key can be present under
//! different namespaces.

use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::TryRuntimeError;

use crate::Config;

pub(crate) fn try_state<T>(n: BlockNumberFor<T>) -> Result<(), TryRuntimeError>
where
	T: Config,
{
	crate::fungible::try_state::check_fungible_consistency::<T>(n)
}
