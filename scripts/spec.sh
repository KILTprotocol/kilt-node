#!/bin/bash
set -x
set -e

TMP_DIR="/tmp/parachain/$USER/"

mkdir -p $TMP_DIR

# build and copy the binary. Make sure we don't rebuild because of changed spec files.
cargo build --release -p kilt-parachain
cp target/release/kilt-parachain $TMP_DIR/kilt-parachain

cargo build --release -p kilt-parachain --features fast-gov
cp target/release/kilt-parachain $TMP_DIR/kilt-parachain-fast-gov

RELAY_CHAIN_IMG=polkadot:fast

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

docker run --entrypoint "/usr/local/bin/polkadot" $RELAY_CHAIN_IMG build-spec --chain rococo-local --disable-default-bootnode >$RELAY_PEREGRINE_PLAIN
$TMP_DIR/kilt-parachain build-spec --runtime spiritnet --chain spiritnet-dev --disable-default-bootnode >$PEREGRINE_PLAIN

python3 scripts/peregrine_relay.py $RELAY_PEREGRINE_PLAIN $RELAY_PEREGRINE
python3 scripts/peregrine_kilt.py $PEREGRINE_PLAIN $PEREGRINE_JQ

docker run --entrypoint "/usr/local/bin/polkadot" -v $(dirname $RELAY_PEREGRINE):/data/spec $RELAY_CHAIN_IMG build-spec --chain /data/spec/$(basename -- "$RELAY_PEREGRINE") --raw --disable-default-bootnode >$RELAY_PEREGRINE_OUT
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
SPIRITNET_OUTPUT=nodes/parachain/res/spiritnet.json

$TMP_DIR/kilt-parachain build-spec --chain spiritnet-new --disable-default-bootnode --raw >$SPIRITNET_OUTPUT

# ##############################################################################
# #                                                                            #
# #                               westend-kilt                                 #
# #                                                                            #
# ##############################################################################
WESTEND_OUTPUT=dev-specs/kilt-parachain/kilt-westend.json

$TMP_DIR/kilt-parachain build-spec --chain wilt-new --disable-default-bootnode --raw >$WESTEND_OUTPUT
