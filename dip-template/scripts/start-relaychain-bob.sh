#!/usr/bin/env sh

/home/antonio/Developer/polkadot/target/debug/polkadot	\
	--bob --validator --base-path /tmp/relay/bob		\
	--chain ../res/rococo-local-0.9.38.raw.json 		\
	--port 40002 --ws-port 50002 --execution wasm
