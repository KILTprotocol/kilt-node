#!/usr/bin/env sh

/home/antonio/Developer/kilt-node/target/debug/dip-node-template	\
	--alice --collator --base-path /tmp/para/sender/alice			\
	--force-authoring --chain dev-sender --port 40010				\
	--ws-port 50010												 	\
	--																\
	--chain ../res/rococo-local-0.9.38.raw.json --execution wasm 	\
	--port 40100 --ws-port 50100
