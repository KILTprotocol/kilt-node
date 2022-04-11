#!/bin/env fish
# Check a few feature combinations for all crates.
# Requires `cargo-workspaces` to be installed.

for features in "--features default" "--all-features" "--features runtime-benchmarks" "--features try-runtime"
	for package in (cargo workspaces list)
		cargo build -p $package (echo $features | string split " ") > /dev/null ^ /dev/null
		if [ "$status" = "0" ]
			echo -n "[ok]   "
		else
			echo -n "[fail] "
		end
		echo cargo build -p $package (echo $features | string split " ")
	end
end
