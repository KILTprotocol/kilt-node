import { test, beforeAll, afterAll } from 'vitest'
import { connectParachains, connectVertical } from '@acala-network/chopsticks'
import { sendTransaction } from '@acala-network/chopsticks-testing'

import * as SpiritnetNetwork from '../network/spiritnet'
import * as PolkadotNetwork from '../network/polkadot'
import * as HydraDxNetwork from '../network/hydroDx'
import type { Config } from '../network/types'
import { keysBob } from '../helper'

let spiritnetContext: Config
let hydradxContext: Config
let polkadotContext: Config

beforeAll(async () => {
	spiritnetContext = await SpiritnetNetwork.getContext()
	hydradxContext = await HydraDxNetwork.getContext()
	polkadotContext = await PolkadotNetwork.getContext()

	await polkadotContext.dev.setStorage(PolkadotNetwork.defaultStorage)
	await spiritnetContext.dev.setStorage(SpiritnetNetwork.defaultStorage)

	// Setup network
	await connectVertical(polkadotContext.chain, spiritnetContext.chain)
	await connectVertical(polkadotContext.chain, hydradxContext.chain)
	await connectParachains([spiritnetContext.chain, hydradxContext.chain])

	let blocks = await polkadotContext.dev.newBlock({ count: 10 })
	console.log(blocks)
	await spiritnetContext.dev.newBlock()

	console.log('Runtime Upgrade completed')
}, 60_000)

afterAll(() => {
	spiritnetContext.teardown()
	hydradxContext.teardown()
	polkadotContext.teardown()
})

test(
	'Limited Reserve Transfers from Spiritnet Account Bob -> HydraDx',
	async () => {
		const remakrkTx = spiritnetContext.api.tx.system.remarkWithEvent('hello').signAsync(keysBob)

		const { events } = await sendTransaction(remakrkTx)

		await spiritnetContext.chain.newBlock()

		console.log((await events).map((ev) => ev.toHuman()))

		const signedTx = spiritnetContext.api.tx.polkadotXcm
			.limitedReserveTransferAssets(
				{
					V3: {
						parents: 1,
						interior: {
							X1: {
								Parachain: HydraDxNetwork.paraId,
							},
						},
					},
				},
				{
					V3: {
						parents: 1,
						interior: {
							X1: {
								AccountId32: {
									id: HydraDxNetwork.sovereignAccount,
								},
							},
						},
					},
				},
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

		console.log((await (await signedTx).paymentInfo(keysBob.address)).toHuman())
		const tx0 = await sendTransaction(signedTx)

		console.log(tx0)

		await spiritnetContext.chain.newBlock()
	},
	{ timeout: 240000 }
)
