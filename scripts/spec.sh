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

cargo build --release -p kilt-parachain --features fast-gov
cp target/release/kilt-parachain $TMP_DIR/kilt-parachain-fast-gov

# ##############################################################################
# #                                                                            #
# #                                  PEREGRINE                                 #
# #                                                                            #
# ##############################################################################
RELAY_PEREGRINE_PLAIN=$TMP_DIR"rococo.plain.json"
RELAY_PEREGRINE=$TMP_DIR"rococo.json"
RELAY_PEREGRINE_OUT=dev-specs/kilt-parachain/peregrine-relay.json

PEREGRINE_PLAIN=$TMP_DIR"peregrine-kilt.plain.spec"
PEREGRINE_JQ=$TMP_DIR"peregrine-kilt.json"
PEREGRINE_OUTPUT=dev-specs/kilt-parachain/peregrine-kilt.json

docker run parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain rococo-local --disable-default-bootnode > $RELAY_PEREGRINE_PLAIN
$TMP_DIR/kilt-parachain-fast-gov build-spec --runtime mashnet --chain dev --disable-default-bootnode > $PEREGRINE_PLAIN

jq -f scripts/peregrine-relay.jq $RELAY_PEREGRINE_PLAIN > $RELAY_PEREGRINE
jq -f scripts/peregrine-kilt.jq $PEREGRINE_PLAIN > $PEREGRINE_JQ

docker run -v $(dirname $RELAY_PEREGRINE):/data/spec parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain /data/spec/$(basename -- "$RELAY_PEREGRINE") --raw --disable-default-bootnode > $RELAY_PEREGRINE_OUT
$TMP_DIR/kilt-parachain-fast-gov build-spec --runtime mashnet --chain $PEREGRINE_JQ --disable-default-bootnode --raw > $PEREGRINE_OUTPUT

ROCOCO_PLAIN=/tmp/parachain/rococo.plain.json
ROCOCO=/tmp/parachain/rococo.json

# ##############################################################################
# #                                                                            #
# #                                  PEREGRINE                                 #
# #                                                                            #
# ##############################################################################
docker run parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain rococo-local --disable-default-bootnode > $ROCOCO_PLAIN
cargo run --release -p kilt-parachain --features fast-gov -- build-spec --chain peregrine --disable-default-bootnode > peregrine-kilt.plain.spec

jq -f scripts/peregrine-relay.jq $ROCOCO_PLAIN >$ROCOCO
jq -f scripts/peregrine-kilt.jq peregrine-kilt.plain.spec >peregrine-kilt.json

docker run -v $(dirname $ROCOCO):/data/spec parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain /data/spec/$(basename -- "$ROCOCO") --raw --disable-default-bootnode > dev-specs/kilt-parachain/peregrine-relay.json
cargo run --release -p kilt-parachain --features fast-gov -- build-spec --chain peregrine-kilt.json --disable-default-bootnode > dev-specs/kilt-parachain/peregrine-kilt.json

# ##############################################################################
# #                                                                            #
# #                               ROCOCO STAGING                               #
# #                                                                            #
# ##############################################################################
RELAY_STAGING_PLAIN=$TMP_DIR"rococo.plain.json"
RELAY_STAGING_JQ=$TMP_DIR"rococo.json"
RELAY_STAGING_OUT=dev-specs/kilt-parachain/relay-stage.json

STAGING_PLAIN=$TMP_DIR"staging-kilt.plain.spec"
STAGING_JQ=$TMP_DIR"staging.json"
STAGING_OUTPUT=dev-specs/kilt-parachain/kilt-stage.json

docker run parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain rococo-local --disable-default-bootnode >$RELAY_STAGING_PLAIN
$TMP_DIR/kilt-parachain build-spec --runtime mashnet --chain staging --disable-default-bootnode > $STAGING_PLAIN

jq -f scripts/roc-stage-relay.jq $RELAY_STAGING_PLAIN > $RELAY_STAGING_JQ
jq -f scripts/roc-stage-kilt.jq $STAGING_PLAIN > $STAGING_JQ

docker run -v$(dirname $RELAY_STAGING_JQ):/data/spec parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain /data/spec/$(basename -- "$RELAY_STAGING_JQ") --raw --disable-default-bootnode > $RELAY_STAGING_OUT
$TMP_DIR/kilt-parachain build-spec --runtime mashnet --chain $STAGING_JQ --disable-default-bootnode --raw > $STAGING_OUTPUT

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

jq -f scripts/kilt-spiritnet.jq $SPIRITNET_PLAIN >$SPIRITNET_JQ

$TMP_DIR/kilt-parachain build-spec --runtime spiritnet --chain $SPIRITNET_JQ --disable-default-bootnode --raw >$SPIRITNET_OUTPUT
