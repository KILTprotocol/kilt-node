import { beforeAll, afterAll } from 'vitest'
import { connectParachains, connectVertical } from '@acala-network/chopsticks'
import { setTimeout } from 'timers/promises'

import * as SpiritnetConfig from '../network/spiritnet.js'
import * as PolkadotConfig from '../network/polkadot.js'
import * as HydraDxConfig from '../network/hydraDx.js'
import * as AssetHubConfig from '../network/assetHub.js'
import type { Config } from '../network/types.js'

export let spiritnetContext: Config
export let hydradxContext: Config
export let polkadotContext: Config
export let assetHubContext: Config

beforeAll(async () => {
	spiritnetContext = await SpiritnetConfig.getContext()
	hydradxContext = await HydraDxConfig.getContext()
	polkadotContext = await PolkadotConfig.getContext()
	assetHubContext = await AssetHubConfig.getContext()

	// Setup network

	await connectVertical(polkadotContext.chain, spiritnetContext.chain)
	await connectVertical(polkadotContext.chain, hydradxContext.chain)
	await connectVertical(polkadotContext.chain, assetHubContext.chain)
	await connectParachains([spiritnetContext.chain, hydradxContext.chain, assetHubContext.chain])

	const newBlockConfig = { count: 2 }
	// fixes api runtime disconnect warning
	await setTimeout(50)
	// Perform runtime upgrade and establish xcm connections.
	await Promise.all([
		polkadotContext.dev.newBlock(newBlockConfig),
		spiritnetContext.dev.newBlock(newBlockConfig),
		hydradxContext.dev.newBlock(newBlockConfig),
		assetHubContext.dev.newBlock(newBlockConfig),
	])
}, 300_000)

afterAll(async () => {
	try {
		await setTimeout(50)
		await Promise.all([
			spiritnetContext.teardown(),
			hydradxContext.teardown(),
			polkadotContext.teardown(),
			assetHubContext.teardown(),
		])
	} catch (e) {
		console.error(e)
	}
})

export async function getFreeBalanceSpiritnet(account: string): Promise<bigint> {
	const accountInfo = await spiritnetContext.api.query.system.account(account)
	return accountInfo.data.free.toBigInt()
}

export async function getFreeBalanceHydraDxKilt(account: string): Promise<bigint> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const accountInfo: any = await hydradxContext.api.query.tokens.accounts(account, HydraDxConfig.kiltTokenId)
	return accountInfo.free.toBigInt()
}
