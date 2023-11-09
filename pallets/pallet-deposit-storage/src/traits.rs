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

use crate::{Config, DepositEntryOf, DepositKeyOf};

pub trait DepositStorageHooks<Runtime>
where
	Runtime: Config,
{
	type Error: Into<u16>;

	fn on_deposit_reclaimed(
		namespace: &Runtime::Namespace,
		key: &DepositKeyOf<Runtime>,
		deposit: DepositEntryOf<Runtime>,
	) -> Result<(), Self::Error>;
}

pub struct NoopDepositStorageHooks;

impl<Runtime> DepositStorageHooks<Runtime> for NoopDepositStorageHooks
where
	Runtime: Config,
{
	type Error = u16;

	fn on_deposit_reclaimed(
		_namespace: &Runtime::Namespace,
		_key: &DepositKeyOf<Runtime>,
		_deposit: DepositEntryOf<Runtime>,
	) -> Result<(), Self::Error> {
		Ok(())
	}
}
