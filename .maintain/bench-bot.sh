#!/bin/bash
# Initially based on https://github.com/paritytech/bench-bot/blob/cd3b2943d911ae29e41fe6204788ef99c19412c3/bench.js

# Most external variables used in this script, such as $GH_CONTRIBUTOR, are
# related to https://github.com/paritytech/try-runtime-bot

# This script relies on $GITHUB_TOKEN which is probably a protected GitLab CI
# variable; if this assumption holds true, it is implied that this script should
# be ran only on protected pipelines

set -eu -o pipefail
shopt -s inherit_errexit

. "$(dirname "${BASH_SOURCE[0]}")/utils.sh"
. "$(dirname "${BASH_SOURCE[0]}")/cmd_runner.sh"

cargo_run_benchmarks="cargo +nightly run --quiet --profile=production"
repository="$(basename "$PWD")"

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
        --json-file="${ARTIFACTS_DIR}/bench.json"
        --header=./file_header.txt
      )

      local output_file=""
      if [[ $pallet == *"::"* ]]; then
        # translates e.g. "pallet_foo::bar" to "pallet_foo_bar"
        output_file="${pallet//::/_}.rs"
      fi

      case "$kind" in
        pallet)
          args+=(
            --output="./parachains/runtimes/$chain_type/$runtime/src/weights/${output_file}"
          )
        ;;
        xcm)
          mkdir -p "./parachains/runtimes/$chain_type/$runtime/src/weights/xcm"
          args+=(
            --template=./templates/xcm-bench-template.hbs
            --output="./parachains/runtimes/$chain_type/$runtime/src/weights/xcm/${output_file}"
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

  $cargo_run_benchmarks "${args[@]}"
}

process_args() {
  local subcommand="$1"
  shift

  case "$subcommand" in
    runtime|pallet|xcm)
      bench_pallet "$subcommand" "$@"
    ;;
    *)
      die "Invalid subcommand $subcommand to process_args"
    ;;
  esac
}

main() {
  cmd_runner_setup

  # Remove the "github" remote since the same repository might be reused by a
  # GitLab runner, therefore the remote might already exist from a previous run
  # in case it was not cleaned up properly for some reason
  &>/dev/null git remote remove github || :

  tmp_dirs=()
  cleanup() {
    exit_code=$?
    # Clean up the "github" remote at the end since it contains the
    # $GITHUB_TOKEN secret, which is only available for protected pipelines on
    # GitLab
    &>/dev/null git remote remove github || :
    rm -rf "${tmp_dirs[@]}"
    exit $exit_code
  }
  trap cleanup EXIT

  if [[
    "${UPSTREAM_MERGE:-}" != "n" &&
    ("${GH_OWNER_BRANCH:-}")
  ]]; then
    echo "Merging $GH_OWNER/$GH_OWNER_REPO#$GH_OWNER_BRANCH into $GH_CONTRIBUTOR_BRANCH"
    git remote add \
      github \
      "https://token:${GITHUB_TOKEN}@github.com/${GH_OWNER}/${GH_OWNER_REPO}.git"
    git pull --no-edit github "$GH_OWNER_BRANCH"
    git remote remove github
  fi

  # shellcheck disable=SC2119
  cmd_runner_apply_patches

  set -x
  # Runs the command to generate the weights
  process_args "$@"
  set +x

  # in case we used diener to patch some dependency during benchmark execution,
  # revert the patches so that they're not included in the diff
  git checkout --quiet HEAD Cargo.toml

  # Save the generated weights to GitLab artifacts in case commit+push fails
  echo "Showing weights diff for command"
  git diff -P | tee -a "${ARTIFACTS_DIR}/weights.patch"
  echo "Wrote weights patch to \"${ARTIFACTS_DIR}/weights.patch\""

  # Commits the weights and pushes it
  git add .
  git commit -m "$COMMIT_MESSAGE"

  # Push the results to the target branch
  git remote add \
    github \
    "https://token:${GITHUB_TOKEN}@github.com/${GH_CONTRIBUTOR}/${GH_CONTRIBUTOR_REPO}.git"
  git push github "HEAD:${GH_CONTRIBUTOR_BRANCH}"
}

main "$@"
