#!/bin/bash

aws configure set region eu-central-1

if [[ "$1" == "spiritnet" ]]; then
aws s3 cp /tmp/spiritnet-metadata.json s3://$S3_BUCKET/spiritnet/$2/
else
aws s3 cp /tmp/peregrine-metadata.json s3://$S3_BUCKET/peregrine/$2/
fi
