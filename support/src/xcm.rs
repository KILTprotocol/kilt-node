// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use sp_std::marker::PhantomData;
use sp_weights::Weight;
use xcm::v4::{Location, Response, XcmContext};
use xcm_executor::traits::OnResponse;

const LOG_TARGET: &str = "xcm::either-or";

// The `OnResponse` trait is not implemented for generic tuples, so we need to
// define our own type to do that, as otherwise we hit Rust's orphan rule.
pub struct EitherOr<A, B>(PhantomData<(A, B)>);

impl<A, B> OnResponse for EitherOr<A, B>
where
	A: OnResponse,
	B: OnResponse,
{
	fn expecting_response(origin: &Location, query_id: u64, querier: Option<&Location>) -> bool {
		A::expecting_response(origin, query_id, querier) || B::expecting_response(origin, query_id, querier)
	}

	fn on_response(
		origin: &Location,
		query_id: u64,
		querier: Option<&Location>,
		response: Response,
		max_weight: Weight,
		context: &XcmContext,
	) -> Weight {
		if A::expecting_response(origin, query_id, querier) {
			log::trace!(target: LOG_TARGET, "Forwarding action to handler A.");
			A::on_response(origin, query_id, querier, response, max_weight, context)
		} else if B::expecting_response(origin, query_id, querier) {
			log::trace!(target: LOG_TARGET, "Forwarding action to handler B.");
			B::on_response(origin, query_id, querier, response, max_weight, context)
		} else {
			log::trace!(target: LOG_TARGET, "Neither A nor B handle this response.");
			Weight::zero()
		}
	}
}
