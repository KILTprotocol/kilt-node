#!/usr/bin/env bash
BASEDIR=$(realpath $(dirname "$0"))
set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
   rustup install nightly-2022-05-11
   rustup update stable
fi
rustup target add wasm32-unknown-unknown --toolchain nightly-2022-05-11
rustup target add wasm32-unknown-unknown --toolchain stable
rustup override set nightly-2022-05-11 --path $BASEDIR/..
