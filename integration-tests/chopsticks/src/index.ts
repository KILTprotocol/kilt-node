import { connectParachains, connectVertical } from '@acala-network/chopsticks'

import * as SpiritnetNetwork from './network/spiritnet.js'
import * as PolkadotNetwork from './network/polkadot.js'
import * as HydraDxNetwork from './network/hydroDx.js'
import { keysCharlie, keysBob } from './helper.js'

async function spinUpNetwork() {
	const spiritnetContext = await SpiritnetNetwork.getContext()
	const hydradxContext = await HydraDxNetwork.getContext()
	const polkadotContext = await PolkadotNetwork.getContext()

	await polkadotContext.dev.setStorage(PolkadotNetwork.defaultStorage(keysCharlie.address))
	await spiritnetContext.dev.setStorage(SpiritnetNetwork.defaultStorage(keysBob.address))

	// Setup network
	await connectVertical(polkadotContext.chain, spiritnetContext.chain)
	await connectVertical(polkadotContext.chain, hydradxContext.chain)
	await connectParachains([spiritnetContext.chain, hydradxContext.chain])
}

spinUpNetwork()
