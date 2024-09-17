#!/bin/bash

# Runs all benchmarks for all pallets, for a given runtime, provided by $1
# Should be run on a reference machine to gain accurate benchmarks
# current Substrate reference machine: https://github.com/paritytech/substrate/pull/5848

runtime=${1-"peregrine"}
profile=${2-"dev"}

chain=$([ "$1" == "spiritnet" ] && echo "spiritnet-dev" || echo "dev")
# Dev profile is the debug target
standard_args="--profile $2 --locked --features=runtime-benchmarks --bin=kilt-parachain"

pallets=(
	pallet-migration
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
	pallet-xcm
	parachain-staking
	public-credentials
	pallet-deposit-storage
	pallet-dip-provider
	pallet-message-queue
	cumulus-pallet-parachain-system
	pallet_multisig
	pallet-assets
	pallet-asset-switch
)

# Add Peregrine-only pallets here!
if [ "$runtime" = "peregrine" ]; then
  pallets+=(
	pallet-sudo
  )
fi

echo "[+] Running all runtime benchmarks for \"$runtime\", \"--chain=$chain\" and profile \"$profile\""

cargo build $standard_args

if [ $profile == "dev" ]; then
    target_folder="debug"
	# We care about benchmark correctness, not accuracy.
	additional_args="--steps=1 --repeat=1 --default-pov-mode=ignored --no-verify"
else
    target_folder=$profile
	additional_args=""
fi

for pallet in "${pallets[@]}"; do
	echo "Runtime: $runtime. Pallet: $pallet"
	# shellcheck disable=SC2086
	./target/$target_folder/kilt-parachain benchmark pallet \
		--template=".maintain/runtime-weight-template.hbs" \
		--header="HEADER-GPL" \
		--wasm-execution=compiled \
		--heap-pages=4096 \
		--chain="${chain}" \
		--pallet="$pallet" \
		--extrinsic="*" \
		--output="./runtimes/${runtime}/src/weights/" \
		$additional_args

done
