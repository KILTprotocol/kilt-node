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

use frame_support::{assert_noop, assert_ok, crypto::ecdsa::ECDSAExt, traits::fungible::InspectHold};
use kilt_support::{mock::mock_origin, Deposit};
use parity_scale_codec::Encode;
use sha3::{Digest, Keccak256};
use sp_runtime::{
	app_crypto::{ecdsa, sr25519, Pair},
	traits::{IdentifyAccount, Zero},
	MultiSignature, MultiSigner, TokenError,
};

use crate::{
	account::{AccountId20, EthereumSignature},
	associate_account_request::{get_challenge, AssociateAccountRequest},
	linkable_account::LinkableAccountId,
	mock::*,
	signature::get_wrapped_payload,
	ConnectedAccounts, ConnectedDids, ConnectionRecord, Error, HoldReason,
};





 


