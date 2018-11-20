#!/usr/bin/env bash

set -e

echo "obtain the project folder"
PROJECT_ROOT=`pwd`
#PROJECT_ROOT=`git rev-parse --show-toplevel`

export CARGO_INCREMENTAL=0

# Save current directory.
pushd .

for SRC in runtime/wasm
do
  echo "$PROJECT_ROOT/$SRC"
  echo "*** Building wasm binaries in $SRC"
  cd "$PROJECT_ROOT/$SRC"

  chmod a+x build.sh
  ./build.sh

  cd - >> /dev/null
done

# Restore initial directory.
popd
