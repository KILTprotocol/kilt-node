import { SetupConfig } from '@acala-network/chopsticks-testing'

import { setStorage, setupNetwork, shutDownNetwork } from '../network/utils.js'
import { BasicConfig } from './types.js'

export async function spinUpNetwork({ network, storage }: BasicConfig) {
	const { parachains, relay } = network

	const { parachainContexts, relayChainContext } = await setupNetwork(relay, parachains)
	const [senderChainContext, receiverChainContext] = parachainContexts

	const { receiverStorage, senderStorage, relayStorage } = storage
	await setStorage(senderChainContext, senderStorage)
	await setStorage(receiverChainContext, receiverStorage)
	await setStorage(relayChainContext, relayStorage)

	return { receiverChainContext, senderChainContext, relayChainContext }
}

export async function tearDownNetwork(chains: SetupConfig[]) {
	try {
		await shutDownNetwork(chains)
	} catch (error) {
		if (!(error instanceof TypeError)) {
			console.error(error)
		}
	}
}
