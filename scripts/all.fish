#!/bin/env fish
# Check a few feature combinations for all crates.
# Requires `cargo-workspaces` to be installed.

echo "Features check started..."

for features in "--features default" "--all-features" "--features runtime-benchmarks" "--features try-runtime"
	for package in (cargo workspaces list)
		cargo clippy -p $package --all-targets (echo $features | string split " ") > /dev/null ^ /dev/null
		if [ "$status" = "0" ]
			echo -n "[ok]   "
		else
			echo -n "[fail] "
		end
		echo cargo clippy -p $package --all-targets (echo $features | string split " ")
	end
end

echo "Features check completed!"
