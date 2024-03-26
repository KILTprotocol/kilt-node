import { test, beforeAll, afterAll } from 'vitest'
import { connectParachains, connectVertical, xcmLogger } from '@acala-network/chopsticks'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as SpiritnetNetwork from '../network/spiritnet.js'
import * as PolkadotNetwork from '../network/polkadot.js'
import * as HydraDxNetwork from '../network/hydroDx.js'
import type { Config } from '../network/types.js'
import { keysBob, keysCharlie } from '../helper.js'

let spiritnetContext: Config
let hydradxContext: Config
let polkadotContext: Config

beforeAll(async () => {
	xcmLogger.level = 'info'
	spiritnetContext = await SpiritnetNetwork.getContext()
	hydradxContext = await HydraDxNetwork.getContext()
	polkadotContext = await PolkadotNetwork.getContext()

	await polkadotContext.dev.setStorage(PolkadotNetwork.defaultStorage(keysCharlie.address))
	await spiritnetContext.dev.setStorage(SpiritnetNetwork.defaultStorage(keysBob.address))
	await hydradxContext.dev.setStorage(HydraDxNetwork.defaultStorage(keysBob.address))

	// Setup network
	await connectVertical(polkadotContext.chain, spiritnetContext.chain)
	await connectVertical(polkadotContext.chain, hydradxContext.chain)
	await connectParachains([spiritnetContext.chain, hydradxContext.chain])

	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 500))
	// Perform runtime upgrade
	await Promise.all([polkadotContext.dev.newBlock(), spiritnetContext.dev.newBlock(), hydradxContext.dev.newBlock()])
	console.log('Runtime Upgrade completed')
}, 40_000)

afterAll(async () => {
	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 500))
	await Promise.all([spiritnetContext.teardown(), hydradxContext.teardown(), polkadotContext.teardown()])
})

test('Limited Reserve Transfers from Spiritnet Account Bob -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(
			SpiritnetNetwork.spiritnet.hydraDxDestination,
			SpiritnetNetwork.spiritnet.hydraDxBeneficiary,
			{
				V2: [
					{
						id: { Concrete: { parents: 0, interior: 'Here' } },
						fun: { Fungible: 1 * 10e12 },
					},
				],
			},
			0,
			'Unlimited'
		)
		.signAsync(keysBob)

	const events = await sendTransaction(signedTx)

	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 50))
	await spiritnetContext.chain.newBlock()
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')

	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 50))
	await hydradxContext.dev.newBlock()

	checkSystemEvents(hydradxContext, 'tokens').toMatchSnapshot('receiver events tokens')
	checkSystemEvents(hydradxContext, 'currencies').toMatchSnapshot('receiver events currencies')
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')
}, 20_000)
