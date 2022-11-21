#!/bin/bash

# This file is separated to simplify testing the final pure command output

set -eu -o pipefail
shopt -s inherit_errexit

. "$(dirname "${BASH_SOURCE[0]}")/utils.sh"
. "$(dirname "${BASH_SOURCE[0]}")/cmd_runner.sh"

cargo_run_benchmarks="cargo run --locked --quiet --profile=production"
current_folder="$(basename "$PWD")"

get_arg optional --repo "$@"
repository="${out:=$current_folder}"

echo "Repo: $repository"

cargo_run() {
  echo "Running $cargo_run_benchmarks" "${args[@]}"

  $cargo_run_benchmarks "${args[@]}"
}

bench_pallet_common_args=(
  --
  benchmark
  pallet
  --steps=50
  --repeat=20
  --extrinsic="*"
  --execution=wasm
  --wasm-execution=compiled
  --heap-pages=4096
  --json-file="${ARTIFACTS_DIR}/bench.json"
)
bench_pallet() {
  local kind="$1"
  local runtime="$2"

  local args
  case "$repository" in
    substrate)
      local pallet="$3"

      args=(
        --features=runtime-benchmarks
        --manifest-path=bin/node/cli/Cargo.toml
        "${bench_pallet_common_args[@]}"
        --pallet="$pallet"
        --chain="$runtime"
      )

      case "$kind" in
        pallet)
          # Translates e.g. "pallet_foo::bar" to "pallet_foo_bar"
          local output_dir="${pallet//::/_}"

          # Substrate benchmarks are output to the "frame" directory but they aren't
          # named exactly after the $pallet argument. For example:
          # - When $pallet == pallet_balances, the output folder is frame/balances
          # - When $pallet == frame_benchmarking, the output folder is frame/benchmarking
          # The common pattern we infer from those examples is that we should remove
          # the prefix
          if [[ "$output_dir" =~ ^[A-Za-z]*[^A-Za-z](.*)$ ]]; then
            output_dir="${BASH_REMATCH[1]}"
          fi

          # We also need to translate '_' to '-' due to the folders' naming
          # conventions
          output_dir="${output_dir//_/-}"

          args+=(
            --header="./HEADER-APACHE2"
            --output="./frame/$output_dir/src/weights.rs"
            --template=./.maintain/frame-weight-template.hbs
          )
        ;;
        *)
          die "Kind $kind is not supported for $repository in bench_pallet"
        ;;
      esac
    ;;
    polkadot)
      local pallet="$3"

      args=(
        --features=runtime-benchmarks
        "${bench_pallet_common_args[@]}"
        --pallet="$pallet"
        --chain="$runtime"
      )

      local runtime_dir
      if [ "$runtime" == dev ]; then
        runtime_dir=polkadot
      elif [[ "$runtime" =~ ^(.*)-dev$  ]]; then
        runtime_dir="${BASH_REMATCH[1]}"
      else
        die "Could not infer weights directory from $runtime"
      fi
      local weights_dir="./runtime/${runtime_dir}/src/weights"

      case "$kind" in
        runtime)
          args+=(
            --header=./file_header.txt
            --output="${weights_dir}/"
          )
        ;;
        xcm)
          args+=(
            --header=./file_header.txt
            --template=./xcm/pallet-xcm-benchmarks/template.hbs
            --output="${weights_dir}/xcm/"
          )
        ;;
        *)
          die "Kind $kind is not supported for $repository in bench_pallet"
        ;;
      esac
    ;;
  kilt-node)
  	  local pallet="$3"

  	  args=(
  		--features=runtime-benchmarks
  		"${bench_pallet_common_args[@]}"
  		--pallet="$pallet"
  		--chain="$runtime"
  	  )

  	  local runtime_dir
  	  if [ "$runtime" == dev ]; then
  		runtime_dir=peregrine
  	  elif [[ "$runtime" =~ ^(.*)-dev$  ]]; then
  		runtime_dir="spiritnet"
  	  else
  		die "Could not infer weights directory from $runtime"
  	  fi
  	  local weights_dir="./runtimes/${runtime_dir}/src/weights"

  	  local output_file=""
  	  if [[ $pallet == *"::"* ]]; then
  		# translates e.g. "pallet_foo::bar" to "pallet_foo_bar"
  		output_file="${pallet//::/_}.rs"
  	  fi

  	  case "$kind" in
  		runtime)
  		  args+=(
  			--header=./file_header.txt
  			--output="${weights_dir}/${output_file}"
  		  )
  		;;
      pallet)
        # We also need to translate '_' to '-' due to the folders' naming
        # conventions
        output_dir="${output_dir//_/-}"

        args+=(
          --output="./pallets/$output_dir/src/default_weights.rs"
          --template=./.maintain/weight-template.hbs
        )
      ;;
      *)
        die "Kind $kind is not supported for $repository in bench_pallet"
      ;;
  		xcm)
  		  args+=(
  			--template=./xcm/pallet-xcm-benchmarks/template.hbs
  			--output="${weights_dir}/xcm/${output_file}"
  		  )
  		;;
  		*)
  		  die "Kind $kind is not supported for $repository in bench_pallet"
  		;;
  	  esac
  	;;
    cumulus)
      local chain_type="$3"
      local pallet="$4"

      args=(
        --bin=polkadot-parachain
        --features=runtime-benchmarks
        "${bench_pallet_common_args[@]}"
        --pallet="$pallet"
        --chain="${runtime}-dev"
        --header=./file_header.txt
      )

      case "$kind" in
        pallet)
          args+=(
            --output="./parachains/runtimes/$chain_type/$runtime/src/weights/"
          )
        ;;
        xcm)
          mkdir -p "./parachains/runtimes/$chain_type/$runtime/src/weights/xcm"
          args+=(
            --template=./templates/xcm-bench-template.hbs
            --output="./parachains/runtimes/$chain_type/$runtime/src/weights/xcm/"
          )
        ;;
        *)
          die "Kind $kind is not supported for $repository in bench_pallet"
        ;;
      esac
    ;;
    *)
      die "Repository $repository is not supported in bench_pallet"
    ;;
  esac

  cargo_run "${args[@]}"
}


bench_overhead_common_args=(
  --
  benchmark
  overhead
  --execution=wasm
  --wasm-execution=compiled
  --warmup=10
  --repeat=100
)
bench_overhead() {
  local args
  case "$repository" in
    substrate)
      args=(
        "${bench_overhead_common_args[@]}"
        --header=./HEADER-APACHE2
        --weight-path="./frame/support/src/weights"
        --chain="dev"
      )
    ;;
    polkadot)
      local runtime="$2"
      args=(
        "${bench_overhead_common_args[@]}"
        --header=./file_header.txt
        --weight-path="./runtime/$runtime/constants/src/weights"
        --chain="$runtime-dev"
      )
    ;;
    cumulus)
      local chain_type="$2"
      local runtime="$3"

      args=(
        --bin=polkadot-parachain
        "${bench_overhead_common_args[@]}"
        --header=./file_header.txt
        --weight-path="./cumulus/parachains/runtimes/$chain_type/$runtime/src/weights"
        --chain="$runtime"
      )
    ;;
    *)
      die "Repository $repository is not supported in bench_overhead"
    ;;
  esac

  cargo_run "${args[@]}"
}

process_args() {
  local subcommand="$1"
  shift

  case "$subcommand" in
    runtime|pallet|xcm)
      echo 'Running bench_pallet'
      bench_pallet "$subcommand" "$@"
    ;;
    overhead)
      echo 'Running bench_overhead'
      bench_overhead "$subcommand" "$@"
    ;;
    *)
      die "Invalid subcommand $subcommand to process_args"
    ;;
  esac
}

process_args "$@"
