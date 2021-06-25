#!/bin/bash
set -x
set -e

TMP_DIR="/tmp/parachain/$USER/"

mkdir -p $TMP_DIR

# build and copy the binary. Make sure we don't rebuild because of changed spec files.
cargo build --release -p kilt-parachain
cp target/release/kilt-parachain $TMP_DIR/kilt-parachain

# cargo build --release -p kilt-parachain --features fast-gov
# cp target/release/kilt-parachain $TMP_DIR/kilt-parachain-fast-gov

RELAY_CHAIN_IMG=parity/polkadot:v0.9.5
RELAY_BINARY="/usr/bin/polkadot"

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

docker run --entrypoint $RELAY_BINARY $RELAY_CHAIN_IMG build-spec --chain rococo-local --disable-default-bootnode >$RELAY_PEREGRINE_PLAIN
$TMP_DIR/kilt-parachain build-spec --runtime peregrine --chain dev --disable-default-bootnode >$PEREGRINE_PLAIN

python3 scripts/peregrine_relay.py $RELAY_PEREGRINE_PLAIN $RELAY_PEREGRINE
python3 scripts/peregrine_kilt.py $PEREGRINE_PLAIN $PEREGRINE_JQ

docker run --entrypoint $RELAY_BINARY -v $(dirname $RELAY_PEREGRINE):/data/spec $RELAY_CHAIN_IMG build-spec --chain /data/spec/$(basename -- "$RELAY_PEREGRINE") --raw --disable-default-bootnode >$RELAY_PEREGRINE_OUT
$TMP_DIR/kilt-parachain build-spec --runtime peregrine --chain $PEREGRINE_JQ --disable-default-bootnode --raw >$PEREGRINE_OUTPUT

# ##############################################################################
# #                                                                            #
# #                              PEREGRINE-STG                                 #
# #                                                                            #
# ##############################################################################
RELAY_PEREGRINE_STG_PLAIN=$TMP_DIR"westend.plain.json"
RELAY_PEREGRINE_STG=$TMP_DIR"westend.json"
RELAY_PEREGRINE_STG_OUT=dev-specs/kilt-parachain/peregrine-stg-relay.json

PEREGRINE_STG_PLAIN=$TMP_DIR"peregrine-stg-kilt.plain.spec"
PEREGRINE_STG_JQ=$TMP_DIR"peregrine-stg-kilt.json"
PEREGRINE_STG_OUTPUT=dev-specs/kilt-parachain/peregrine-stg-kilt.json

docker run --entrypoint $RELAY_BINARY $RELAY_CHAIN_IMG build-spec --chain westend-local --disable-default-bootnode >$RELAY_PEREGRINE_STG_PLAIN
python3 scripts/peregrine_stg_relay.py $RELAY_PEREGRINE_STG_PLAIN $RELAY_PEREGRINE_STG
docker run --entrypoint $RELAY_BINARY -v $(dirname $RELAY_PEREGRINE_STG):/data/spec $RELAY_CHAIN_IMG build-spec --chain /data/spec/$(basename -- "$RELAY_PEREGRINE_STG") --raw --disable-default-bootnode >$RELAY_PEREGRINE_STG_OUT

$TMP_DIR/kilt-parachain build-spec --runtime peregrine --chain dev --disable-default-bootnode >$PEREGRINE_STG_PLAIN
python3 scripts/peregrine_stg_kilt.py $PEREGRINE_STG_PLAIN $PEREGRINE_STG_JQ
$TMP_DIR/kilt-parachain build-spec --runtime peregrine --chain $PEREGRINE_STG_JQ --disable-default-bootnode --raw >$PEREGRINE_STG_OUTPUT

# ##############################################################################
# #                                                                            #
# #                         PEREGRINE-DEV Fast-Gov                             #
# #                                                                            #
# ##############################################################################
# PEREGRINE_FG_PLAIN=$TMP_DIR"peregrine-kilt-dev-fast-gov.plain.spec"
# PEREGRINE_FG_JQ=$TMP_DIR"peregrine-kilt-dev-fast-gov.json"
# PEREGRINE_FG_OUTPUT=dev-specs/kilt-parachain/peregrine-kilt-dev-fast-gov.json

# $TMP_DIR/kilt-parachain-fast-gov build-spec --runtime peregrine --chain dev --disable-default-bootnode >$PEREGRINE_FG_PLAIN

# python3 scripts/peregrine_kilt_dev.py $PEREGRINE_FG_PLAIN $PEREGRINE_FG_JQ

# $TMP_DIR/kilt-parachain-fast-gov build-spec --runtime peregrine --chain $PEREGRINE_FG_JQ --disable-default-bootnode --raw >$PEREGRINE_FG_OUTPUT

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
