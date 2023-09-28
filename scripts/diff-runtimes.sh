#!/bin/bash

GITLAB_TOKE=
PROJECT_ID="26909212"
BRANCH_NAME="develop"
JOB_NAME_SPIRIT="build-wasm-spiritnet"
JOB_NAME_PEREGRINE="build-wasm-peregrine"

SPIRITNET_DIR=artifacts-spirit
PEREGRINE_DIR=artifacts-pere

curl -o artifacts-spirit.zip -L --raw --header "Private-Token: ${GITLAB_TOKEN}" "https://gitlab.com/api/v4/projects/${PROJECT_ID}/jobs/artifacts/${BRANCH_NAME}/download?job=${JOB_NAME_SPIRIT}"
curl -o artifacts-pere.zip -L --raw --header "Private-Token: ${GITLAB_TOKEN}" "https://gitlab.com/api/v4/projects/${PROJECT_ID}/jobs/artifacts/${BRANCH_NAME}/download?job=${JOB_NAME_PEREGRINE}"

unzip -u artifacts-spirit.zip -d artifacts-spirit
unzip -u artifacts-pere.zip -d artifacts-pere

cargo build --release -p spiritnet-runtime
cargo build --release -p peregrine-runtime

subwasm diff --no-color $SPIRITNET_DIR/out/spiritnet_runtime.compact.compressed.wasm target/release/wbuild/spiritnet-runtime/spiritnet_runtime.compact.compressed.wasm | tee develop-diff-spiritnet.txt
subwasm diff --no-color $PEREGRINE_DIR/out/peregrine_runtime.compact.compressed.wasm target/release/wbuild/peregrine-runtime/peregrine_runtime.compact.compressed.wasm | tee develop-diff-peregrine.txt
