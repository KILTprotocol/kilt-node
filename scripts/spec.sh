#!/bin/bash
set -x
set -e

TMP_DIR="/tmp/parachain/$USER/"

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
# cargo build --release -p kilt-parachain
# cp target/release/kilt-parachain $TMP_DIR/kilt-parachain

# cargo build --release -p kilt-parachain --features fast-gov
# cp target/release/kilt-parachain $TMP_DIR/kilt-parachain-fast-gov

RELAY_CHAIN_IMG=parity/polkadot:v0.9.3

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

docker run $RELAY_CHAIN_IMG build-spec --chain rococo-local --disable-default-bootnode >$RELAY_PEREGRINE_PLAIN
$TMP_DIR/kilt-parachain build-spec --runtime spiritnet --chain spiritnet-dev --disable-default-bootnode >$PEREGRINE_PLAIN

python3 scripts/peregrine_relay.py $RELAY_PEREGRINE_PLAIN $RELAY_PEREGRINE
python3 scripts/peregrine_kilt.py $PEREGRINE_PLAIN $PEREGRINE_JQ

docker run -v $(dirname $RELAY_PEREGRINE):/data/spec $RELAY_CHAIN_IMG build-spec --chain /data/spec/$(basename -- "$RELAY_PEREGRINE") --raw --disable-default-bootnode >$RELAY_PEREGRINE_OUT
$TMP_DIR/kilt-parachain build-spec --runtime spiritnet --chain $PEREGRINE_JQ --disable-default-bootnode --raw >$PEREGRINE_OUTPUT

# ##############################################################################
# #                                                                            #
# #                         PEREGRINE Mashnet Fast-Gov                         #
# #                                                                            #
# ##############################################################################
PEREGRINE_FG_PLAIN=$TMP_DIR"peregrine-kilt-fast-gov.plain.spec"
PEREGRINE_FG_JQ=$TMP_DIR"peregrine-kilt-fast-gov.json"
PEREGRINE_FG_OUTPUT=dev-specs/kilt-parachain/peregrine-kilt-fast-gov.json

$TMP_DIR/kilt-parachain-fast-gov build-spec --runtime mashnet --chain dev --disable-default-bootnode >$PEREGRINE_FG_PLAIN

python3 scripts/peregrine_kilt.py $PEREGRINE_FG_PLAIN $PEREGRINE_FG_JQ

$TMP_DIR/kilt-parachain-fast-gov build-spec --runtime mashnet --chain $PEREGRINE_FG_JQ --disable-default-bootnode --raw >$PEREGRINE_FG_OUTPUT

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

python3 scripts/spiritnet_kilt.py $SPIRITNET_PLAIN $SPIRITNET_JQ

$TMP_DIR/kilt-parachain build-spec --runtime spiritnet --chain $SPIRITNET_JQ --disable-default-bootnode --raw >$SPIRITNET_OUTPUT

# ##############################################################################
# #                                                                            #
# #                               westend-kilt                                 #
# #                                                                            #
# ##############################################################################
WESTEND_PLAIN=$TMP_DIR"kilt-westend.plain.json"
WESTEND_JQ=$TMP_DIR"kilt-westend.json"
WESTEND_OUTPUT=dev-specs/kilt-parachain/kilt-westend.json

WESTEND_RELAY_PLAIN=$TMP_DIR"westend-relay.plain.json"
WESTEND_RELAY_JQ=$TMP_DIR"westend-relay.json"
WESTEND_RELAY_OUTPUT=dev-specs/kilt-parachain/westend-relay.json

docker run $RELAY_CHAIN_IMG build-spec --chain westend-local --disable-default-bootnode >$WESTEND_RELAY_PLAIN
$TMP_DIR/kilt-parachain build-spec --chain spiritnet-dev --disable-default-bootnode >$WESTEND_PLAIN

python3 scripts/westend_relay.py $WESTEND_RELAY_PLAIN $WESTEND_RELAY_JQ
python3 scripts/westend_kilt.py $WESTEND_PLAIN $WESTEND_JQ

docker run -v$(dirname $WESTEND_RELAY_JQ):/data/spec $RELAY_CHAIN_IMG build-spec --chain /data/spec/$(basename -- "$WESTEND_RELAY_JQ") --raw --disable-default-bootnode >$WESTEND_RELAY_OUTPUT
$TMP_DIR/kilt-parachain build-spec --runtime spiritnet --chain $WESTEND_JQ --disable-default-bootnode --raw >$WESTEND_OUTPUT
