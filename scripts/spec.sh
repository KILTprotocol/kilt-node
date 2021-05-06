#!/bin/bash

# ##############################################################################
# #                                                                            #
# #                               ROCOCO STAGING                               #
# #                                                                            #
# ##############################################################################
docker run parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain rococo-local --disable-default-bootnode >rococo.plain.json
cargo run --release -p kilt-parachain --features fast-gov -- build-spec --chain staging --disable-default-bootnode >kilt-stage.plain.json

jq -f scripts/roc-stage-relay.jq rococo.plain.json >rococo.json
jq -f scripts/roc-stage-kilt.jq kilt-stage.plain.json >kilt-stage.json

docker run -v $PWD:/data/spec parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain /data/spec/rococo.json --raw --disable-default-bootnode >dev-specs/kilt-parachain/relay-stage.json
cargo run --release -p kilt-parachain --features fast-gov -- build-spec --chain kilt-stage.json --disable-default-bootnode >dev-specs/kilt-parachain/kilt-stage.json
