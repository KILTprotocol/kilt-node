#!/bin/bash

# Runs all benchmarks for all pallets, for a given runtime, provided by $1
# Should be run on a reference machine to gain accurate benchmarks
# current Substrate reference machine: https://github.com/paritytech/substrate/pull/5848

runtime="$1"
chain=$([ "$1" == "spiritnet" ] && echo "spiritnet-dev" || echo "dev")
standard_args="--release --locked --features=runtime-benchmarks --bin=kilt-parachain"

pallets=(
    attestation
    ctype
    delegation
    did
    kilt_launch
    parachain_staking
)

echo "[+] Running all benchmarks for $runtime --chain=$chain"

for pallet in "${pallets[@]}"; do
    echo "Runtime: $runtime. Pallet: $pallet";
    # shellcheck disable=SC2086
    cargo +nightly run $standard_args -- benchmark \
    --chain="${chain}" \
    --steps=50 \
    --repeat=20 \
    --pallet="$pallet" \
    --extrinsic="*" \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output="./runtimes/${runtime}/src/weights/${pallet}.rs" \
    --template=".maintain/runtime-weight-template.hbs"
done
