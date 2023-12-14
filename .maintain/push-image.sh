#!/bin/bash

source_tag=$1
target_tag=$2

PROVIDER_BIN_NAME="dip-provider-node-template"
CONSUMER_BIN_NAME="dip-consumer-node-template"

# publish to docker hub
docker tag local/standalone-node:$source_tag ${DOCKER_HUB_STANDALONE}:$target_tag
docker tag local/kilt-node:$source_tag ${DOCKER_HUB_PARACHAIN}:$target_tag
docker tag local/$PROVIDER_BIN_NAME:$source_tag ${DOCKER_HUB_DIP_PROVIDER_TEMPLATE}:$target_tag
docker tag local/$CONSUMER_BIN_NAME:$source_tag ${DOCKER_HUB_DIP_CONSUMER_TEMPLATE}:$target_tag

docker push ${DOCKER_HUB_STANDALONE}:$target_tag
docker push ${DOCKER_HUB_PARACHAIN}:$target_tag
docker push ${DOCKER_HUB_DIP_PROVIDER_TEMPLATE}:$target_tag
docker push ${DOCKER_HUB_DIP_CONSUMER_TEMPLATE}:$target_tag

# publish to AWS
docker tag local/standalone-node:$source_tag $AWS_REGISTRY/kilt/prototype-chain:$target_tag
docker tag local/kilt-node:$source_tag $AWS_REGISTRY/kilt-parachain/collator:$target_tag
docker tag local/$PROVIDER_BIN_NAME:$source_tag $AWS_REGISTRY/$PROVIDER_BIN_NAME:$target_tag
docker tag local/$CONSUMER_BIN_NAME:$source_tag $AWS_REGISTRY/$CONSUMER_BIN_NAME:$target_tag

docker push $AWS_REGISTRY/kilt/prototype-chain:$target_tag
docker push $AWS_REGISTRY/kilt-parachain/collator:$target_tag
docker push $AWS_REGISTRY/$PROVIDER_BIN_NAME:$target_tag
docker push $AWS_REGISTRY/$CONSUMER_BIN_NAME:$target_tag
