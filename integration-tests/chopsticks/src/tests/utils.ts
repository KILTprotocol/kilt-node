import { SetupConfig } from '@acala-network/chopsticks-testing'

import { createBlock, scheduleTx, setStorage, setupNetwork, shutDownNetwork } from '../network/utils.js'
import { BasicConfig } from './types.js'

export async function spinUpNetwork({ network, storage, setUpTx }: BasicConfig) {
	const { parachains, relay } = network
	const { parachainContexts, relayChainContext } = await setupNetwork(relay, parachains)
	const [senderChainContext, receiverChainContext] = parachainContexts

	const {
		parachains: [senderStorage, receiverStorage],
		relay: relayStorage,
	} = storage

	if (senderChainContext) {
		await setStorage(senderChainContext, senderStorage)
	}

	if (receiverChainContext) {
		await setStorage(receiverChainContext, receiverStorage)
	}

	if (relayChainContext) {
		await setStorage(relayChainContext, relayStorage)
	}

	if (setUpTx) {
		await Promise.all(
			setUpTx.map(async ([tx, chain]) => {
				if (chain === 'receiver') {
					const rawTx = tx(receiverChainContext)
					await scheduleTx(receiverChainContext, rawTx)
					await createBlock(receiverChainContext)
				}
				if (chain === 'sender') {
					const rawTx = tx(senderChainContext)
					await scheduleTx(senderChainContext, rawTx)
					await createBlock(senderChainContext)
				}
				if (chain === 'relay') {
					const rawTx = tx(relayChainContext)
					await scheduleTx(relayChainContext, rawTx)
					await createBlock(relayChainContext)
				}
			})
		)
	}

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
