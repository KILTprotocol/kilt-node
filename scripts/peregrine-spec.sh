#!/bin/bash

docker run parity/rococo:rococo-v1 build-spec --chain rococo-local --disable-default-bootnode > rococo.plain.json

jq -f scripts/peregrine-relay.jq rococo.plain.json > rococo.json

docker run -v $PWD:/data/spec parity/rococo:rococo-v1 build-spec --chain /data/spec/rococo.json --raw --disable-default-bootnode > rococo.raw.json
