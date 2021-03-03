#!/usr/bin/env bash
BASEDIR=$(dirname "$0")
set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
   rustup install nightly-2021-02-22
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain 2021-02-22
rustup override set nightly-2021-02-22 --path $BASEDIR/..
