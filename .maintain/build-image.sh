#!/bin/bash

target_tag=$1

# build parachain image and standalone image
docker build --cache-from $AWS_REGISTRY/kilt-parachain/collator:$target_tag --build-arg NODE_TYPE=kilt-parachain -t local/kilt-node:$target_tag .
docker build --cache-from $AWS_REGISTRY/kilt/prototype-chain:$target_tag --build-arg NODE_TYPE=mashnet-node -t local/mashnet-node:$target_tag .
