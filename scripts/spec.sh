#!/bin/bash

mkdir -p /tmp/parachain/

# ##############################################################################
# #                                                                            #
# #                               ROCOCO STAGING                               #
# #                                                                            #
# ##############################################################################
docker run parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain rococo-local --disable-default-bootnode >/tmp/parachain/rococo.plain.json
cargo run --release -p kilt-parachain -- build-spec --chain staging --disable-default-bootnode >/tmp/parachain/kilt-stage.plain.json

jq -f scripts/roc-stage-relay.jq /tmp/parachain/rococo.plain.json >/tmp/parachain/rococo.json
jq -f scripts/roc-stage-kilt.jq /tmp/parachain/kilt-stage.plain.json >/tmp/parachain/kilt-stage.json

docker run -v /tmp/parachain/:/data/spec parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain /data/spec/rococo.json --raw --disable-default-bootnode >dev-specs/kilt-parachain/relay-stage.json
cargo run --release -p kilt-parachain -- build-spec --chain /tmp/parachain/kilt-stage.json --disable-default-bootnode >dev-specs/kilt-parachain/kilt-stage.json
