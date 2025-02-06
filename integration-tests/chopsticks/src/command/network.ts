import { mainChains } from '../network/index.js'
import { setStorage, setupNetwork } from '../network/utils.js'

export async function createTestNetwork() {
	const relayConfig = mainChains.polkadot.getConfig({})

	// 1. Create the network specific state
	const kiltStorage = mainChains.kilt.storage.assignNativeTokensToAccounts([
		'4pF5Y2Eo6doQHPLQj5AkndZwtomVB8ab2sRftRS2D9JDdELr',
	])

	// 2. Create the network specific config
	const kiltConfig = mainChains.kilt.getConfig({})

	// 2. Add the different parachainContexts here. The order of parachainContexts and parachainStorage should match.
	const parachainOptions = [kiltConfig]
	const parachainStorage = [kiltStorage]

	const { relayChainContext, parachainContexts } = await setupNetwork(relayConfig, parachainOptions)

	await Promise.all(parachainStorage.map((storage, index) => setStorage(parachainContexts[index], storage)))

	await Promise.all([...parachainContexts, relayChainContext].map((context) => context.pause()))
}
