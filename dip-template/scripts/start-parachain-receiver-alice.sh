#!/usr/bin/env sh

/home/antonio/Developer/kilt-node/target/debug/dip-node-template	\
	--alice --collator --base-path /tmp/para/receiver/alice			\
	--force-authoring --chain dev-receiver --port 40060				\
	--ws-port 50060 -lruntime=debug									\
	--																\
	--chain ../res/rococo-local-0.9.38.raw.json --execution wasm 	\
	--port 40600 --ws-port 50600
