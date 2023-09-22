#!/bin/bash

target_tag=$1

docker build \
    --target builder \
    --cache-from $AWS_REGISTRY/kilt-parachain/collator:builder \
    -t $AWS_REGISTRY/kilt-parachain/collator:builder \
    .
docker push $AWS_REGISTRY/kilt-parachain/collator:builder

# build parachain image and standalone image
docker build \
    --cache-from $AWS_REGISTRY/kilt-parachain/collator:builder \
    --cache-from $AWS_REGISTRY/kilt-parachain/collator:$target_tag \
    --build-arg NODE_TYPE=kilt-parachain \
    -t local/kilt-node:$target_tag \
    .
docker build \
    --cache-from $AWS_REGISTRY/kilt-parachain/collator:builder \
    --cache-from $AWS_REGISTRY/kilt/prototype-chain:$target_tag \
    --build-arg NODE_TYPE=standalone-node \
    -t local/standalone-node:$target_tag \
    .
