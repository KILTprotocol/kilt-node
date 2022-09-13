#!/bin/bash

export ENCODED_PASSWORD= `echo -n "AWS:$(aws ecr get-login-password --region $AWS_DEFAULT_REGION)" | base64`
export PAYLOAD=`jq -n --arg userpass $ENCODED_PASSWORD '{"auths": {"'$AWS_REGISTRY'": {"auth": $userpass}}}'`

curl --request PUT --header "PRIVATE-TOKEN:$TOKEN" "https://gitlab.com/api/v4/projects/26909212/variables/DOCKER_AUTH_CONFIG" --form "value=$PAYLOAD"
