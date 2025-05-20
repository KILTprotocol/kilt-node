#!/bin/bash
set -x

# Runs all benchmarks for all pallets, for a given runtime, provided by $1
# Should be run on a reference machine to gain accurate benchmarks
# current Substrate reference machine: https://github.com/paritytech/substrate/pull/5848

runtime=${1-"peregrine"}
profile=${2-"release"}

# Dev profile is the debug target
standard_args="--profile $profile --locked --features=runtime-benchmarks --package $runtime-runtime"

pallets=(
	pallet-migration
	attestation
	ctype
	delegation
	did
	frame-system
	pallet-balances
	pallet-democracy
	pallet-indices
	pallet-preimage
	pallet-proxy
	pallet-scheduler
	pallet-session
	pallet-timestamp
	pallet-tips
	pallet-treasury
	pallet-utility
	pallet-vesting
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
	# `pallet-membership` instances
	pallet-membership
	pallet-technical-membership
	pallet-collators
	# `pallet-collective` instances
	pallet-collective
	pallet-technical-committee-collective
	# `pallet-did-lookup` instances
	pallet-did-lookup
	# `pallet-web3-names` instances
	pallet-web3-names
	# ISMP
	ismp-parachain
	pallet-token-gateway
)

# Add Peregrine-only pallets here!
if [ "$runtime" = "peregrine" ]; then
	pallets+=(
		pallet-sudo
		pallet-bonded-assets
		pallet-bonded-coins
	)
fi

echo "[+] Running all runtime benchmarks for \"$runtime\", and profile \"$profile\""

cargo build $standard_args

if [ "$profile" = "dev" ]; then
	target_folder="debug"
	file_extension=".wasm"
	# We care about benchmark correctness, not accuracy.
	additional_args="--steps=2 --repeat=1 --default-pov-mode=ignored"
else
	target_folder=$profile
	file_extension=".compact.compressed.wasm"
	additional_args="--header=HEADER-GPL --template=.maintain/runtime-weight-template.hbs --output=./runtimes/${runtime}/src/weights/"
fi

wasm_path="./target/$target_folder/wbuild/$runtime-runtime/${runtime}_runtime$file_extension"

for pallet in "${pallets[@]}"; do
	echo "Runtime: $runtime. Pallet: $pallet"
	# shellcheck disable=SC2086
	frame-omni-bencher v1 benchmark pallet \
		--pallet="$pallet" \
		--extrinsic="*" \
		--genesis-builder="runtime" \
		--runtime=$wasm_path \
		$additional_args

	bench_status=$?

	# Exit with error as soon as one benchmark fails
	if [ $bench_status -ne 0 ]; then
		exit $bench_status
	fi

done
