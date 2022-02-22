#!/bin/bash

source_tag = $1
target_tag = $2

# publish to docker hub
docker tag kiltprotocol/mashnet-node:$source_tag ${DOCKER_HUB_STANDALONE}:$target_tag
docker tag kiltprotocol/kilt-node:$source_tag ${DOCKER_HUB_PARACHAIN}:$target_tag

docker push ${DOCKER_HUB_STANDALONE}:$target_tag
docker push ${DOCKER_HUB_PARACHAIN}:$target_tag

# publish to AWS
docker tag kiltprotocol/mashnet-node:$source_tag $AWS_REGISTRY/kilt/prototype-chain:$target_tag
docker tag kiltprotocol/kilt-node:$source_tag $AWS_REGISTRY/kilt-parachain/collator:$target_tag

docker push $AWS_REGISTRY/kilt/prototype-chain:$target_tag
docker push $AWS_REGISTRY/kilt-parachain/collator:$target_tag
