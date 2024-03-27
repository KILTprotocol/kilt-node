import { beforeAll, afterAll } from 'vitest'
import { connectParachains, connectVertical, xcmLogger } from '@acala-network/chopsticks'

import * as SpiritnetNetwork from '../network/spiritnet.js'
import * as PolkadotNetwork from '../network/polkadot.js'
import * as HydraDxNetwork from '../network/hydraDx.js'
import type { Config } from '../network/types.js'

export let spiritnetContext: Config
export let hydradxContext: Config
export let polkadotContext: Config

beforeAll(async () => {
	xcmLogger.level = 'info'
	spiritnetContext = await SpiritnetNetwork.getContext()
	hydradxContext = await HydraDxNetwork.getContext()
	polkadotContext = await PolkadotNetwork.getContext()

	// Setup network
	await connectVertical(polkadotContext.chain, spiritnetContext.chain)
	await connectVertical(polkadotContext.chain, hydradxContext.chain)
	await connectParachains([spiritnetContext.chain, hydradxContext.chain])

	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 500))
	// Perform runtime upgrade and establish xcm connections.
	await Promise.all([polkadotContext.dev.newBlock(), spiritnetContext.dev.newBlock(), hydradxContext.dev.newBlock()])
	console.info('Runtime Upgrade completed')
}, 40_000)

afterAll(async () => {
	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 500))
	await Promise.all([spiritnetContext.teardown(), hydradxContext.teardown(), polkadotContext.teardown()])
})

export async function getFreeBalanceSpiritnet(account: string): Promise<number> {
	const accountInfo = await spiritnetContext.api.query.system.account(account)
	return accountInfo.data.free.toNumber()
}

export async function getFreeBalanceHydraDxKilt(account: string): Promise<number> {
	const accountInfo: any = await hydradxContext.api.query.tokens.accounts(account, HydraDxNetwork.kiltTokenId)
	return accountInfo.free.toNumber()
}
