#!/usr/bin/env sh

/home/antonio/Developer/polkadot/target/debug/polkadot	\
	--alice --validator --base-path /tmp/relay/alice	\
	--chain ../res/rococo-local-0.9.38.raw.json 		\
	--port 40001 --ws-port 50001 --execution wasm
