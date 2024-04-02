import { beforeEach, afterEach } from 'vitest'
import { connectParachains, connectVertical, xcmLogger } from '@acala-network/chopsticks'
import { setTimeout } from 'timers/promises'

import * as SpiritnetNetwork from '../network/spiritnet.js'
import * as PolkadotNetwork from '../network/polkadot.js'
import * as HydraDxNetwork from '../network/hydraDx.js'
import type { Config } from '../network/types.js'

export let spiritnetContext: Config
export let hydradxContext: Config
export let polkadotContext: Config

// There is not really a way to reset the storage. dev.setStorage only appends or overwrites an existing entry
beforeEach(async () => {
	xcmLogger.level = 'info'
	spiritnetContext = await SpiritnetNetwork.getContext()
	hydradxContext = await HydraDxNetwork.getContext()
	polkadotContext = await PolkadotNetwork.getContext()

	// Setup network
	await connectVertical(polkadotContext.chain, spiritnetContext.chain)
	await connectVertical(polkadotContext.chain, hydradxContext.chain)
	await connectParachains([spiritnetContext.chain, hydradxContext.chain])

	const newBlockConfig = { count: 2 }
	// fixes api runtime disconnect warning
	await setTimeout(50)
	// Perform runtime upgrade and establish xcm connections.
	await Promise.all([
		polkadotContext.dev.newBlock(newBlockConfig),
		spiritnetContext.dev.newBlock(newBlockConfig),
		hydradxContext.dev.newBlock(newBlockConfig),
	])
	console.info('Runtime Upgrade completed')
}, 60_000)

afterEach(async () => {
	// fixes api runtime disconnect warning
	await setTimeout(50)
	await Promise.all([spiritnetContext.teardown(), hydradxContext.teardown(), polkadotContext.teardown()])
})

export async function getFreeBalanceSpiritnet(account: string): Promise<bigint> {
	const accountInfo = await spiritnetContext.api.query.system.account(account)
	return accountInfo.data.free.toBigInt()
}

export async function getFreeBalanceHydraDxKilt(account: string): Promise<bigint> {
	const accountInfo: any = await hydradxContext.api.query.tokens.accounts(account, HydraDxNetwork.kiltTokenId)
	return accountInfo.free.toBigInt()
}
