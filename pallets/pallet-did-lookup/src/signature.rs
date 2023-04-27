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

use parity_scale_codec::alloc::string::ToString;
use sp_std::vec::Vec;

// According to https://github.com/polkadot-js/common/blob/5d5c7e4c0ace06e3301ccadfd3c3351955f1e251/packages/util/src/u8a/wrap.ts#L13
const PAYLOAD_BYTES_WRAPPER_PREFIX: &[u8; 7] = b"<Bytes>";
const PAYLOAD_BYTES_WRAPPER_POSTFIX: &[u8; 8] = b"</Bytes>";
const ETHEREUM_SIGNATURE_PREFIX: &[u8; 26] = b"\x19Ethereum Signed Message:\n";
pub(crate) enum WrapType {
	Substrate,
	Ethereum,
}

pub(crate) fn get_wrapped_payload(payload: &[u8], wrap_type: WrapType) -> Vec<u8> {
	match wrap_type {
		WrapType::Substrate => PAYLOAD_BYTES_WRAPPER_PREFIX
			.iter()
			.chain(payload.iter())
			.chain(PAYLOAD_BYTES_WRAPPER_POSTFIX.iter())
			.copied()
			.collect(),
		WrapType::Ethereum => ETHEREUM_SIGNATURE_PREFIX
			.iter()
			// eth wrapping also contains the length of the payload
			.chain(payload.len().to_string().as_bytes().iter())
			.chain(payload.iter())
			.copied()
			.collect(),
	}
}
