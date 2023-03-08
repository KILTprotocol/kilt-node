use codec::{Decode, Encode};
use dip_sender::traits::TxBuilder;
use dip_support::VersionedIdentityProofAction;
use xcm::{latest::MultiLocation, DoubleEncoded};

use crate::DidIdentifier;

#[derive(Encode, Decode)]
enum ReceiverParachainCalls {
	#[codec(index = 50)]
	DipReceiver(ReceiverParachainDipReceiverCalls),
}

#[derive(Encode, Decode)]
enum ReceiverParachainDipReceiverCalls {
	#[codec(index = 0)]
	ProcessIdentityAction(VersionedIdentityProofAction<DidIdentifier, [u8; 32]>),
}

pub struct ReceiverParachainTxBuilder;
impl TxBuilder<DidIdentifier, [u8; 32]> for ReceiverParachainTxBuilder {
	type Error = ();

	fn build(
		_dest: MultiLocation,
		action: VersionedIdentityProofAction<DidIdentifier, [u8; 32]>,
	) -> Result<DoubleEncoded<()>, Self::Error> {
		let double_encoded: DoubleEncoded<()> =
			ReceiverParachainCalls::DipReceiver(ReceiverParachainDipReceiverCalls::ProcessIdentityAction(action))
				.encode()
				.into();
		Ok(double_encoded)
	}
}
