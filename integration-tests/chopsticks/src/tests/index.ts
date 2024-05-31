import { connectParachains, connectVertical } from '@acala-network/chopsticks'
import { setTimeout } from 'timers/promises'
import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'

import type { Config } from '../network/types.js'

const newBlock = async (newBlockConfig: { count: number }, contexts: Config[]) => {
	await Promise.all(contexts.map((context) => context.dev.newBlock(newBlockConfig)))
}

export async function connectNetworks(relayChain: Config, parachains: Config[]) {
	for (const parachain of parachains) {
		await connectVertical(relayChain.chain, parachain.chain)
	}

	await connectParachains(parachains.map((parachain) => parachain.chain))

	const newBlockConfig = { count: 2 }
	// fixes api runtime disconnect warning
	await setTimeout(50)
	// Perform runtime upgrade and establish xcm connections.
	await newBlock(newBlockConfig, [relayChain, ...parachains])
}

export async function shutDownNetwork(chains: Config[]) {
	await setTimeout(50)
	const tearDown = chains.map((chain) => chain.teardown())
	await Promise.all(tearDown)
}

export async function setupNetwork(relayChain: SetupOption, sender: SetupOption, receiver: SetupOption) {
	await setTimeout(50)
	const relayChainContext = await setupContext(relayChain)
	const senderChainContext = await setupContext(sender)
	const receiverChainContext = await setupContext(receiver)

	await connectNetworks(relayChainContext, [senderChainContext, receiverChainContext])
	return { relayChainContext, senderChainContext, receiverChainContext }
}
