#!/bin/env bash

cargo_flags=(
	"--target wasm32-unknown-unknown"
	"--no-default-features"
	"--workspace"
	"--exclude kilt-parachain"
	"--exclude standalone-node"
	"--exclude xcm-integration-tests"
	"--exclude 'dip-provider*'"
	"--exclude 'dip-consumer*'"
)

cargo_flags_output=""

for cargo_flag in "${cargo_flags[@]}"; do
	cargo_flags_output+="$cargo_flag "
done

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

rustc_clippy_flags_output=""

for clippy_flag in "${rustc_clippy_flags[@]}"; do
	rustc_clippy_flags_output+="'$clippy_flag' "
done

command="cargo clippy $cargo_flags_output -- $rustc_clippy_flags_output"
echo $command
