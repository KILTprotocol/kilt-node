#!/usr/bin/env sh

/home/antonio/Developer/polkadot/target/debug/polkadot	\
	--dave --validator --base-path /tmp/relay/dave		\
	--chain ../res/rococo-local-0.9.38.raw.json 		\
	--port 40004 --ws-port 50004 --execution wasm
