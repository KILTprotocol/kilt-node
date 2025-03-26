import { SetupConfig } from '@acala-network/chopsticks-testing'

import { createBlock, scheduleTx, setStorage, setupNetwork, shutDownNetwork } from '../network/utils.js'
import { BasicConfig } from './types.js'

export async function spinUpNetwork({ network }: BasicConfig) {
	const { parachains, relay } = network
	const parachainOptions = parachains.map((parachain) => parachain.option)
	const { parachainContexts, relayChainContext } = await setupNetwork(relay.option, parachainOptions)

	await setStorage(relayChainContext, relay.storage)
	await Promise.all(
		relay.setUpTx.map(async (tx) => {
			const rawTx = tx(relayChainContext)
			await scheduleTx(relayChainContext, rawTx.method.toHex())
			await createBlock(relayChainContext)
		})
	)

	await Promise.all(
		parachains.map(async (parachain, index) => {
			// fetch the right context
			const currentContext = parachainContexts[index]
			// set the storage
			await setStorage(currentContext, parachain.storage)

			// schedule txs.
			await Promise.all(
				parachain.setUpTx.map(async (tx) => {
					const rawTx = tx(currentContext)
					await scheduleTx(currentContext, rawTx.method.toHex())
					await createBlock(currentContext)
				})
			)
		})
	)

	return { parachainContexts, relayChainContext }
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
