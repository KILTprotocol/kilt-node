#!/bin/bash
set -x

TMP_DIR="/tmp/parachain/"

mkdir -p $TMP_DIR

# make sure that jq supports big numbers
EXPECTED_BIG_NUM=10000000000000000000000000000
BIG_NUM=$(echo '{"big_test_num":10000000000000000000000000000}' | jq '.big_test_num')
if [[ $BIG_NUM != $EXPECTED_BIG_NUM ]]; then
	echo "your jq doesn't support big numbers."
	echo "Make sure to install the latest git version"
	echo "Got: " $BIG_NUM " Expected: " $EXPECTED_BIG_NUM
	exit 1
fi

# build and copy the binary. Make sure we don't rebuild because of changed spec files.
cargo build --release -p kilt-parachain
cp target/release/kilt-parachain $TMP_DIR/kilt-parachain


# ##############################################################################
# #                                                                            #
# #                               ROCOCO STAGING                               #
# #                                                                            #
# ##############################################################################
docker run parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain rococo-local --disable-default-bootnode >/tmp/parachain/rococo.plain.json
$TMP_DIR/kilt-parachain build-spec --chain staging --disable-default-bootnode >/tmp/parachain/kilt-stage.plain.json

jq -f scripts/roc-stage-relay.jq /tmp/parachain/rococo.plain.json >/tmp/parachain/rococo.json
jq -f scripts/roc-stage-kilt.jq /tmp/parachain/kilt-stage.plain.json >/tmp/parachain/kilt-stage.json

docker run -v /tmp/parachain/:/data/spec parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain /data/spec/rococo.json --raw --disable-default-bootnode >dev-specs/kilt-parachain/relay-stage.json
$TMP_DIR/kilt-parachain build-spec --chain /tmp/parachain/kilt-stage.json --disable-default-bootnode >dev-specs/kilt-parachain/kilt-stage.json

# ##############################################################################
# #                                                                            #
# #                                 SPIRITNET                                  #
# #                                                                            #
# ##############################################################################
SPIRITNET_PLAIN=$TMP_DIR"spiritnet.plain.json"
SPIRITNET_JQ=$TMP_DIR"spiritnet.json"
SPIRITNET_OUTPUT=nodes/parachain/res/spiritnet.json

# we have to load `spiritnet-dev` here since `spiritnet` would just be the content of the file at $SPIRITNET_OUTPUT
$TMP_DIR/kilt-parachain build-spec --chain spiritnet-dev --disable-default-bootnode >$SPIRITNET_PLAIN

jq -f scripts/roc-stage-kilt.jq $SPIRITNET_PLAIN >$SPIRITNET_JQ

$TMP_DIR/kilt-parachain build-spec --runtime spiritnet --chain $SPIRITNET_JQ --disable-default-bootnode >$SPIRITNET_OUTPUT
