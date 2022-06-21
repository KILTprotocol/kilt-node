#!/usr/bin/env bash

TAGS_TO_KEEP=10  #Keep recent 10 tags and delete the rest

for REPOSITORY_NAME in  {DOCKER_HUB_STANDALONE} ${DOCKER_HUB_PARACHAIN} $AWS_REGISTRY/kilt/prototype-chain $AWS_REGISTRY/kilt-parachain/collator
do
    TAGS=$(docker images ${REPOSITORY_NAME} -a -q)

    idx=0
    for TAG in ${TAGS}
    do

        idx=$((idx+1))

        if [[ ${idx} -gt ${TAGS_TO_KEEP} ]]; then

            IMAGE="${REPOSITORY_NAME}:${TAG}"

            echo "Deleting docker rmi -f ${IMAGE} ..."

            docker rmi -f ${IMAGE}

        else

            echo "Skipping ${IMAGE} at ${idx}"

        fi

    done

done
