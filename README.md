# KILT-node &middot; [![tests](https://gitlab.com/kiltprotocol/mashnet-node/badges/develop/pipeline.svg)](https://gitlab.com/kiltprotocol/mashnet-node/-/commits/develop)


<p align="center">
  <img src="/.maintain/media/kilt.png">
</p>

The KILT blockchain is the heart and soul behind KILT Protocol.
It provides the immutable transaction ledger for the various KILT processes in the network.

The nodes use Parity Substrate as the underlying blockchain technology stack, extended with our custom functionality for handling DIDs, CTypes, Attestations and Delegations.


<div align="center">
	<br>
	<a href="https://dev.kilt.io">
		<img src=".maintain/media/header.svg" width="400" height="200" alt="Click here to get to the developer documentation">
	</a>
	<br>
</div>

## Structure

This repository is structured into multiple parts:

* `/crates/`: Rust crates that are not specific to the KILT runtime and can be used in different environments as well.
* `/nodes/parachain`: The rust code for the parachain blockchain node. This will produce an executable that spins up a KILT parachain node.
* `/nodes/parachain`: The rust code for the standalone blockchain node. This will produce an executable that spins up a KILT blockchain node that can run without the need for a relay chain.
* `/pallets/`: contains all pallets that are developed for the KILT blockchain. Pallets MUST NOT depend on runtimes.
* `/runtime-api/`: Crates that provide RuntimeAPI traits specific to the KILT blockchain.
* `/runtime/spiritnet`: the blockchain logic of the Spiritnet.
* `/runtime/peregrine`: the blockchain logic of the Peregrine testnet.
* `/scripts/`: scripts for running benchmarks and setting up the project.
* `/support/`: Common traits and functionality for the KILT pallets.
* `/zombienet/`: Automated setup and testing of parachain setups
