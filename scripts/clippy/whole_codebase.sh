#!/bin/env bash

# Generate and execute a `cargo clippy` command that applies the
# specified lints to the whole codebase (including tests), hence it contains lints that aim at improving code in general following pre-agreeed on conventions.
#
# It can be called without any parameters, or it can take one parameter to specify additional cargo-specific flags, or two parameters to also specific rustc-specific flags.
# If only rustc-specific flags are to be provided, use an empty string `""` as the first parameter.

# ===== Cargo section =====

cargo_flags=(
	# Targets (all)
	"--all-targets"
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
	"-Wclippy::alloc_instead_of_core"
	"-Wclippy::as_underscore"
	"-Wclippy::clone_on_ref_ptr"
	"-Wclippy::decimal_literal_representation"
	"-Wclippy::else_if_without_else"
	"-Wclippy::empty_drop"
	"-Wclippy::empty_structs_with_brackets"
	"-Wclippy::if_then_some_else_none"
	"-Wclippy::impl_trait_in_params"
	"-Wclippy::mixed_read_write_in_expression"
	"-Wclippy::negative_feature_names"
	"-Wclippy::pattern_type_mismatch"
	"-Wclippy::pub_without_shorthand"
	"-Wclippy::redundant_type_annotations"
	"-Wclippy::ref_patterns"
	"-Wclippy::rest_pat_in_fully_bound_structs"
	"-Wclippy::str_to_string"
	"-Wclippy::string_slice"
	"-Wclippy::string_to_string"
	"-Wclippy::unnecessary_self_imports"
	"-Wclippy::unneeded_field_pattern"
	"-Wclippy::wildcard_dependencies"
	# TODO: Add after upgrading to 1.77
	#"-Wclippy::empty_enum_variants_with_brackets"
	# TODO: Add after upgrading to 1.80
	#"-Wclippy::renamed_function_params"
	# TODO: Add after upgrading to 1.81
	#"-Wclippy::cfg_not_test"
	# TODO: Add after upgrading to 1.83
	#"-Wclippy::unused_trait_names"
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
