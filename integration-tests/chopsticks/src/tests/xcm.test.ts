import { test, beforeAll, afterAll } from 'vitest'
import { connectParachains, connectVertical } from '@acala-network/chopsticks'
import { sendTransaction } from '@acala-network/chopsticks-testing'

import * as SpiritnetNetwork from '../network/spiritnet'
import * as PolkadotNetwork from '../network/polkadot'
import * as HydraDxNetwork from '../network/hydroDx'
import type { Config } from '../network/types'
import { checkEvents, keysBob, keysCharlie } from '../helper'

let spiritnetContext: Config
let hydradxContext: Config
let polkadotContext: Config

beforeAll(async () => {
	spiritnetContext = await SpiritnetNetwork.getContext()
	hydradxContext = await HydraDxNetwork.getContext()
	polkadotContext = await PolkadotNetwork.getContext()

	await polkadotContext.dev.setStorage(PolkadotNetwork.defaultStorage(keysCharlie.address))
	await spiritnetContext.dev.setStorage(SpiritnetNetwork.defaultStorage(keysBob.address))

	// Setup network
	await connectVertical(polkadotContext.chain, spiritnetContext.chain)
	await connectVertical(polkadotContext.chain, hydradxContext.chain)
	await connectParachains([spiritnetContext.chain, hydradxContext.chain])

	console.log('Network is created')

	// Perform runtime upgrade

	await polkadotContext.dev.newBlock()
	console.log('polkadot created block')
	await spiritnetContext.dev.newBlock()
	console.log('Spiritnet created block')

	console.log('Runtime Upgrade completed')
}, 30_000)

afterAll(() => {
	spiritnetContext.teardown()
	hydradxContext.teardown()
	polkadotContext.teardown()
})

test('Limited Reserve Transfers from Spiritnet Account Bob -> HydraDx', async () => {
	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(
			SpiritnetNetwork.spiritnet.hydraDxDestination,
			SpiritnetNetwork.spiritnet.hydraDxBeneficiary,
			{
				V3: [
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

	await spiritnetContext.chain.newBlock()

	checkEvents(events, 'balances').toMatchSnapshot('Balance events')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events')
})
