#!/bin/env bash

# Generate and execute a `cargo clippy` command that applies the specified lints only to code that belongs to the runtime
# or that can be included in a runtime (e.g., DIP components) and that does not include anything
# else than runtime code (i.e., no tests).
#
# It can be called without any parameters, or it can take one parameter to specify additional cargo-specific flags, or two parameters to also specific rustc-specific flags.
# If only rustc-specific flags are to be provided, use an empty string `""` as the first parameter.

# ===== Cargo section =====

cargo_flags=(
	# Target triple (only WASM)
	"--target wasm32-unknown-unknown"
	# Targets (only library targets)
	"--lib"
	# Features (no defaults)
	"--no-default-features"
	# Includes (all)
	"--workspace"
	# Excludes (binaries, integration tests and demos)
	"--exclude kilt-parachain"
	"--exclude standalone-node"
	"--exclude xcm-integration-tests"
	"--exclude 'dip-provider*'"
	"--exclude 'dip-consumer*'"
)

additional_cargo_flags=${1-""}

cargo_flags_output=""

for cargo_flag in "${cargo_flags[@]}"; do
	cargo_flags_output+="$cargo_flag "
done

cargo_flags_output+="$additional_cargo_flags"

# ===== Rustc section =====

# Try to keep this list sorted with denies (-D), forbids (-F), and warnings (-W).
# Also all elements within each category.
rustc_clippy_flags=(
	"-Dclippy::arithmetic_side_effects"
	"-Dclippy::as_conversions"
	"-Dclippy::assertions_on_result_states"
	"-Dclippy::cast_possible_wrap"
	"-Dclippy::dbg_macro"
	"-Dclippy::expect_used"
	"-Dclippy::float_arithmetic"
	"-Dclippy::float_cmp_const"
	"-Dclippy::index_refutable_slice"
	"-Dclippy::indexing_slicing"
	"-Dclippy::lossy_float_literal"
	"-Dclippy::panic"
	"-Dclippy::string_slice"
	"-Dclippy::todo"
	"-Dclippy::unimplemented"
	"-Dclippy::unreachable"
	"-Dclippy::unwrap_used"
	"-Funsafe_code"
	"-Wclippy::integer_division"
	"-Wclippy::modulo_arithmetic"
	"-Wclippy::print_stderr"
	"-Wclippy::print_stdout"
)

additional_rustc_flags=${2-""}

rustc_clippy_flags_output=""

for clippy_flag in "${rustc_clippy_flags[@]}"; do
	rustc_clippy_flags_output+="'$clippy_flag' "
done

rustc_clippy_flags_output+="$additional_rustc_flags"

# ===== Whole command section =====

command="cargo clippy --locked $cargo_flags_output -- $rustc_clippy_flags_output"
echo $command

eval $command
