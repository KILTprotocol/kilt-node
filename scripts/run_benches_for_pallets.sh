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
	pallet-configuration
	pallet-did-lookup
	pallet-inflation
	pallet-web3-names
	parachain-staking
	public-credentials
	pallet-asset-switch
)

// Add Peregrine-only pallets here!
if [ "$runtime" = "peregrine" ]; then
	pallets+=()
fi

echo "[+] Running all default weight benchmarks for $runtime --chain=$chain"

cargo build $standard_args

for pallet in "${pallets[@]}"; do
	echo "Runtime: $runtime. Pallet: $pallet"
	# shellcheck disable=SC2086
	./target/release/kilt-parachain benchmark pallet \
		--template=".maintain/weight-template.hbs" \
		--header="HEADER-GPL" \
		--heap-pages=4096 \
		--chain="${chain}" \
		--pallet="$pallet" \
		--extrinsic="*" \
		--output="./pallets/${pallet//_/-}/src/default_weights.rs"
done
