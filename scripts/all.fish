for features in "--all-features" "--features runtime-benchmarks" "--features try-runtime"
	for package in (cargo workspaces list)
		cargo build -p $package $features > /dev/null ^ /dev/null
		if [ "$status" = "0" ]
			echo -n "[ok]   "
		else
			echo -n "[fail] "
		end
		echo cargo build -p $package $features
	end
end
