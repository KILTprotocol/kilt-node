#!/bin/env bash

# Generate and execute a `cargo clippy` command that applies to the whole codebase excluding tests.
# It should include lints that are to be enforced everywhere but in tests, and should not include lints that are specified in the `all_features_all_targets.sh` script, as those are run on the whole codebase, including tests.
#
# It can be called without any parameters, or it can take one parameter to specify additional cargo-specific flags, or two parameters to also specific rustc-specific flags.
# If only rustc-specific flags are to be provided, use an empty string `""` as the first parameter.

# ===== Cargo section =====

cargo_flags=(
	# Targets (no tests)
	"--lib"
	"--bins"
	"--examples"
	"--benches"
	# Features (all)
	"--all-features"
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
	"-Wclippy::default_numeric_fallback"
	"-Wclippy::error_impl_error"
	"-Wclippy::let_underscore_must_use"
	"-Wclippy::let_underscore_untyped"
	"-Wclippy::shadow_reuse"
	"-Wclippy::shadow_same"
	"-Wclippy::shadow_unrelated"
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
