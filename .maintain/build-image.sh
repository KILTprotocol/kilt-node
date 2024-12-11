#!/bin/bash

set -e

target_tag=$1

# Build the builder image and push it in the background
docker build \
    --target builder \
    --cache-from $CI_REGISTRY/kilt-node:builder \
    -t $CI_REGISTRY/kilt-node:builder \
    . &
docker push $CI_REGISTRY/kilt-node:builder &

wait

# Build and tag images in parallel
build_and_tag() {
    local node_type=$1
    local image_name=$2
    local cache_image=$3

    docker build \
        --cache-from $CI_REGISTRY/kilt-node:builder \
        --cache-from $CI_REGISTRY/$cache_image:$target_tag \
        --build-arg NODE_TYPE=$node_type \
        -t local/$image_name:$target_tag \
        .
}

build_and_tag "kilt-parachain" "kilt-node" "kilt-node" &

build_and_tag "standalone-node" "standalone-node" "standalone-node" &

build_and_tag "dip-provider-node-template" "dip-provider-node-template" "kilt-node" &

build_and_tag "dip-consumer-node-template" "dip-consumer-node-template" "kilt-node" &

wait
