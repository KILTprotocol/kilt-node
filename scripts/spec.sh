#!/bin/bash
mkdir -p /tmp/parachain/

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
docker run parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain rococo-local --disable-default-bootnode > $ROCOCO_PLAIN
cargo run --release -p kilt-parachain -- build-spec --chain staging --disable-default-bootnode > /tmp/parachain/kilt-stage.plain.json

jq -f scripts/roc-stage-relay.jq $ROCOCO_PLAIN >/tmp/parachain/$ROCOCO
jq -f scripts/roc-stage-kilt.jq /tmp/parachain/kilt-stage.plain.json >/tmp/parachain/kilt-stage.json

docker run -v $(dirname $ROCOCO):/data/spec parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain /data/spec/$(basename -- "$ROCOCO") --raw --disable-default-bootnode > dev-specs/kilt-parachain/relay-stage.json
cargo run --release -p kilt-parachain -- build-spec --chain /tmp/parachain/kilt-stage.json --disable-default-bootnode > dev-specs/kilt-parachain/kilt-stage.json
