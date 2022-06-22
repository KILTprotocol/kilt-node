#!/bin/bash

# Runs all benchmarks for all pallets, for a given runtime, provided by $1
# Should be run on a reference machine to gain accurate benchmarks
# current Substrate reference machine: https://github.com/paritytech/substrate/pull/5848

runtime=${1-"peregrine"}
chain=$([ "$1" == "spiritnet" ] && echo "spiritnet-dev" || echo "dev")
standard_args="--release --locked --features=runtime-benchmarks --bin=kilt-parachain"

pallets=(
    attestation
    ctype
    delegation
    did
    parachain_staking
    frame_system
    pallet_balances
    pallet_collective
    pallet_democracy
    pallet_indices
    pallet_membership
    pallet_scheduler
    pallet_timestamp
    pallet_treasury
    pallet_utility
    pallet_session
)

echo "[+] Running all benchmarks for $runtime --chain=$chain"

cargo build $standard_args
for pallet in "${pallets[@]}"; do
    echo "Runtime: $runtime. Pallet: $pallet";
    # shellcheck disable=SC2086
    ./target/release/kilt-parachain benchmark \
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
