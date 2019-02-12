#!/usr/bin/env bash

set -e

echo "obtain the project folder"
PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"

export CARGO_INCREMENTAL=0

bold=$(tput bold)
normal=$(tput sgr0)

# Save current directory.
pushd . >/dev/null

for SRC in runtime/wasm
do
  echo "$PROJECT_ROOT/$SRC"
  echo "${bold}Building webassembly binary in $SRC...${normal}"
  cd "$PROJECT_ROOT/$SRC"

  chmod a+x build.sh
  ./build.sh

  cd - >> /dev/null
done

# Restore initial directory.
popd >/dev/null
