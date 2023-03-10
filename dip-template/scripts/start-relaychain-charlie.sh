#!/usr/bin/env sh

/home/antonio/Developer/polkadot/target/debug/polkadot		\
	--charlie --validator --base-path /tmp/relay/charlie	\
	--chain ../res/rococo-local-0.9.38.raw.json 			\
	--port 40003 --ws-port 50003 --execution wasm
