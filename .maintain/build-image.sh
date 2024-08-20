#!/bin/bash

set -e

target_tag=$1

# Build the builder image and push it sequentially
docker build \
    --target builder \
    --cache-from $AWS_REGISTRY/kilt-parachain/collator:builder \
    -t $AWS_REGISTRY/kilt-parachain/collator:builder \
    .

docker push $AWS_REGISTRY/kilt-parachain/collator:builder

build_and_tag() {
    local node_type=$1
    local image_name=$2
    local cache_image=$3

    docker build \
        --cache-from $AWS_REGISTRY/kilt-parachain/collator:builder \
        --cache-from $AWS_REGISTRY/$cache_image:$target_tag \
        --build-arg NODE_TYPE=$node_type \
        -t local/$image_name:$target_tag \
        .
}

build_and_tag "kilt-parachain" "kilt-node" "kilt-parachain/collator"

build_and_tag "standalone-node" "standalone-node" "kilt/prototype-chain"

build_and_tag "dip-provider-node-template" "dip-provider-node-template" "kilt-parachain/collator"

build_and_tag "dip-consumer-node-template" "dip-consumer-node-template" "kilt-parachain/collator"
