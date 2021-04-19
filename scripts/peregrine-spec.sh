#!/bin/bash

docker run parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain rococo-local --disable-default-bootnode > rococo.plain.json
cargo run --release -p kilt-parachain -- build-spec --chain peregrine --disable-default-bootnode > peregrine-kilt.plain.spec

jq -f scripts/peregrine-relay.jq rococo.plain.json > rococo.json
jq -f scripts/peregrine-kilt.jq peregrine-kilt.plain.spec > peregrine-kilt.spec

docker run -v $PWD:/data/spec parity/rococo:rococo-v1-0.8.30-943038a8-f14fa75f build-spec --chain /data/spec/rococo.json --raw --disable-default-bootnode > dev-specs/kilt-parachain/peregrine-relay.json
cargo run --release -p kilt-parachain -- build-spec --chain peregrine-kilt.spec --disable-default-bootnode > dev-specs/kilt-parachain/peregrine-kilt.json
