import { connectParachains, connectVertical } from '@acala-network/chopsticks'
import { setTimeout } from 'timers/promises'
import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'

/**
 * This function is used to shut down a network composed of multiple chains.
 *
 * @param {Config[]} chains - An array of chain configurations that make up the network.
 *
 * @returns {Promise<void>}
 * Returns a Promise that resolves when all chains in the network have been successfully shut down.
 *
 */
export async function shutDownNetwork(chains: Config[]): Promise<void> {
	await setTimeout(50)
	const tearDown = chains.map((chain) => chain?.teardown())
	await Promise.all(tearDown)
}
const newBlock = async (newBlockConfig: { count: number }, contexts: Config[]) => {
	await Promise.all(contexts.map((context) => context.dev.newBlock(newBlockConfig)))
}
async function connectNetworks(relayChain: Config, parachains: Config[]) {
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

/**
 * This function is used to set up a network with a relay chain, a sender, and a receiver.
 *
 * @param {SetupOption} relayChain - The relay chain option for the network setup.
 * @param {SetupOption} parachains - The parachain option for the network setup.
 *
 * @returns {Promise<{relayChainContext: Config, senderChainContext: Config, receiverChainContext: Config}>}
 * An object containing the contexts of the relay chain, sender, and receiver.
 */
export async function setupNetwork(
	relayChain: SetupOption,
	parachains: SetupOption[]
): Promise<{ relayChainContext: Config; parachainContexts: Config[] }> {
	await setTimeout(50)
	const relayChainContext = await setupContext(relayChain)
	const parachainContexts = await Promise.all(parachains.map((parachain) => setupContext(parachain)))

	await connectNetworks(relayChainContext, parachainContexts)
	return { relayChainContext, parachainContexts }
}

/// Creates a new block for the given context
export async function createBlock(context: Config) {
	// fixes api runtime disconnect warning
	await setTimeout(50)
	await context.dev.newBlock()
}

/// sets the storage for the given context.
export async function setStorage(context: Config, storage: Record<string, Record<string, unknown>>) {
	await setTimeout(50)
	await context.dev.setStorage(storage)
	await createBlock(context)
}

export async function scheduleTx({ api }: { api: ApiPromise }, call: SubmittableExtrinsic<'promise'>, at = undefined) {
	const number = at ? at : (await api.rpc.chain.getHeader()).number.toNumber()
	await api.rpc('dev_setStorage', {
		scheduler: {
			agenda: [
				[
					[number + 1],
					[
						{
							call: {
								Inline: call.method.toHex(),
							},
							origin: {
								system: 'Root',
							},
						},
					],
				],
			],
		},
	})
}
