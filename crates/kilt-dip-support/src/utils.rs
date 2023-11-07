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

use pallet_dip_provider::traits::IdentityProvider;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

pub struct CombinedIdentityResult<OutputA, OutputB, OutputC> {
	pub a: OutputA,
	pub b: OutputB,
	pub c: OutputC,
}

impl<OutputA, OutputB, OutputC> From<(OutputA, OutputB, OutputC)>
	for CombinedIdentityResult<OutputA, OutputB, OutputC>
{
	fn from(value: (OutputA, OutputB, OutputC)) -> Self {
		Self {
			a: value.0,
			b: value.1,
			c: value.2,
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputB: Default,
	OutputC: Default,
{
	pub fn from_a(a: OutputA) -> Self {
		Self {
			a,
			b: OutputB::default(),
			c: OutputC::default(),
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputA: Default,
	OutputC: Default,
{
	pub fn from_b(b: OutputB) -> Self {
		Self {
			a: OutputA::default(),
			b,
			c: OutputC::default(),
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputA: Default,
	OutputB: Default,
{
	pub fn from_c(c: OutputC) -> Self {
		Self {
			a: OutputA::default(),
			b: OutputB::default(),
			c,
		}
	}
}

pub struct CombineIdentityFrom<A, B, C>(PhantomData<(A, B, C)>);

#[derive(Encode, Decode, TypeInfo)]
pub enum CombineError<ErrorA, ErrorB, ErrorC> {
	A(ErrorA),
	B(ErrorB),
	C(ErrorC),
}

impl<ErrorA, ErrorB, ErrorC> From<CombineError<ErrorA, ErrorB, ErrorC>> for u16
where
	ErrorA: Into<u16>,
	ErrorB: Into<u16>,
	ErrorC: Into<u16>,
{
	fn from(value: CombineError<ErrorA, ErrorB, ErrorC>) -> Self {
		match value {
			CombineError::A(error) => error.into(),
			CombineError::B(error) => error.into(),
			CombineError::C(error) => error.into(),
		}
	}
}

impl<Runtime, A, B, C> IdentityProvider<Runtime> for CombineIdentityFrom<A, B, C>
where
	Runtime: pallet_dip_provider::Config,
	A: IdentityProvider<Runtime>,
	B: IdentityProvider<Runtime>,
	C: IdentityProvider<Runtime>,
{
	type Error = CombineError<A::Error, B::Error, C::Error>;
	type Identity = CombinedIdentityResult<Option<A::Identity>, Option<B::Identity>, Option<C::Identity>>;

	fn retrieve(identifier: &Runtime::Identifier) -> Result<Option<Self::Identity>, Self::Error> {
		match (
			A::retrieve(identifier),
			B::retrieve(identifier),
			C::retrieve(identifier),
		) {
			// If no details is returned, return None for the whole result
			(Ok(None), Ok(None), Ok(None)) => Ok(None),
			// Otherwise, return `Some` or `None` depending on each result
			(Ok(ok_a), Ok(ok_b), Ok(ok_c)) => Ok(Some(CombinedIdentityResult {
				a: ok_a,
				b: ok_b,
				c: ok_c,
			})),
			(Err(e), _, _) => Err(CombineError::A(e)),
			(_, Err(e), _) => Err(CombineError::B(e)),
			(_, _, Err(e)) => Err(CombineError::C(e)),
		}
	}
}

pub type OutputOf<Hasher> = <Hasher as sp_runtime::traits::Hash>::Output;
