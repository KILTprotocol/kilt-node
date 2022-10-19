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
    frame-system
    pallet-balances
    pallet-collective
    pallet-democracy
	pallet-did-lookup
    pallet-indices
	pallet-inflation
    pallet-membership
    pallet-preimage
	pallet-proxy
    pallet-scheduler
    pallet-session
    pallet-timestamp
	pallet-tips
    pallet-treasury
    pallet-utility
    pallet-vesting
	pallet-web3-names
    parachain-staking
)

echo "[+] Running all runtime benchmarks for $runtime --chain=$chain"

cargo build $standard_args

for pallet in "${pallets[@]}"; do
    echo "Runtime: $runtime. Pallet: $pallet";
    # shellcheck disable=SC2086
    ./target/release/kilt-parachain benchmark pallet \
    --chain="${chain}" \
    --steps=50 \
    --repeat=20 \
    --pallet="$pallet" \
    --extrinsic="*" \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output="./runtimes/${runtime}/src/weights/${pallet//-/_}.rs" \
    --template=".maintain/runtime-weight-template.hbs"
done
