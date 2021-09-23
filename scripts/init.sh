#!/usr/bin/env bash
BASEDIR=$(realpath $(dirname "$0"))
set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
   rustup install nightly-2021-07-29
   rustup update stable
fi
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-07-29
rustup target add wasm32-unknown-unknown --toolchain stable
rustup override set nightly-2021-07-29 --path $BASEDIR/..
