// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use codec::{Decode, Encode};
use sp_std::str;

use crate::*;

/// The expected URI scheme for HTTP endpoints.
pub const HTTP_URI_SCHEME: &str = "http://";
/// The expected URI scheme for HTTPS endpoints.
pub const HTTPS_URI_SCHEME: &str = "https://";
/// The expected URI scheme for FTP endpoints.
pub const FTP_URI_SCHEME: &str = "ftp://";
/// The expected URI scheme for FTPS endpoints.
pub const FTPS_URI_SCHEME: &str = "ftps://";
/// The expected URI scheme for IPFS endpoints.
pub const IPFS_URI_SCHEME: &str = "ipfs://";

/// A web URL starting with either http:// or https://
/// and containing only ASCII URL-encoded characters.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct HttpUrl {
	payload: Vec<u8>,
}

impl TryFrom<&[u8]> for HttpUrl {
	type Error = UrlError;

	// It fails if the byte sequence does not result in an ASCII-encoded string or
	// if the resulting string contains characters that are not allowed in a URL.
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let str_url = str::from_utf8(value).map_err(|_| UrlError::InvalidUrlEncoding)?;

		ensure!(
			str_url.starts_with(HTTP_URI_SCHEME) || str_url.starts_with(HTTPS_URI_SCHEME),
			UrlError::InvalidUrlScheme
		);

		ensure!(utils::is_valid_ascii_url(&str_url), UrlError::InvalidUrlEncoding);

		Ok(HttpUrl {
			payload: value.to_vec(),
		})
	}
}

/// An FTP URL starting with ftp:// or ftps://
/// and containing only ASCII URL-encoded characters.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct FtpUrl {
	payload: Vec<u8>,
}

impl TryFrom<&[u8]> for FtpUrl {
	type Error = UrlError;

	// It fails if the byte sequence does not result in an ASCII-encoded string or
	// if the resulting string contains characters that are not allowed in a URL.
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let str_url = str::from_utf8(value).map_err(|_| UrlError::InvalidUrlEncoding)?;

		ensure!(
			str_url.starts_with(FTP_URI_SCHEME) || str_url.starts_with(FTPS_URI_SCHEME),
			UrlError::InvalidUrlScheme
		);

		ensure!(utils::is_valid_ascii_url(&str_url), UrlError::InvalidUrlEncoding);

		Ok(FtpUrl {
			payload: value.to_vec(),
		})
	}
}

/// An IPFS URL starting with ipfs://. Both CIDs v0 and v1 supported.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct IpfsUrl {
	payload: Vec<u8>,
}

impl TryFrom<&[u8]> for IpfsUrl {
	type Error = UrlError;

	// It fails if the URL is not ASCII-encoded or does not start with the expected
	// URL scheme.
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let str_url = str::from_utf8(value).map_err(|_| UrlError::InvalidUrlEncoding)?;

		ensure!(str_url.starts_with(IPFS_URI_SCHEME), UrlError::InvalidUrlScheme);

		// Remove the characters of the URL scheme
		let slice_to_verify = str_url
			.get(IPFS_URI_SCHEME.len()..)
			.expect("The minimum length was ensured with starts_with.");

		// Verify the rest are either only base58 or only base32 characters (according
		// to the IPFS specification, respectively versions 0 and 1).
		ensure!(
			utils::is_base_32(slice_to_verify) || utils::is_base_58(slice_to_verify),
			UrlError::InvalidUrlEncoding
		);

		Ok(IpfsUrl {
			payload: value.to_vec(),
		})
	}
}

/// Supported URLs.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub enum Url {
	/// See [HttpUrl].
	Http(HttpUrl),
	/// See [FtpUrl].
	Ftp(FtpUrl),
	/// See [IpfsUrl].
	Ipfs(IpfsUrl),
}

impl From<HttpUrl> for Url {
	fn from(url: HttpUrl) -> Self {
		Self::Http(url)
	}
}

impl From<FtpUrl> for Url {
	fn from(url: FtpUrl) -> Self {
		Self::Ftp(url)
	}
}

impl From<IpfsUrl> for Url {
	fn from(url: IpfsUrl) -> Self {
		Self::Ipfs(url)
	}
}
