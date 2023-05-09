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

use sp_runtime::traits::{CheckedAdd, One, Zero};

pub trait Bump {
	fn bump(&mut self);
}

impl<T> Bump for T
where
	T: CheckedAdd + Zero + One,
{
	// FIXME: Better implementation?
	fn bump(&mut self) {
		if let Some(new) = self.checked_add(&Self::one()) {
			*self = new;
		} else {
			*self = Self::zero();
		}
	}
}

pub trait DidDipOriginFilter<Call> {
	type Error;
	type OriginInfo;
	type Success;

	fn check_call_origin_info(call: &Call, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error>;
}
