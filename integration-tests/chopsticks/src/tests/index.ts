import { afterEach, beforeAll, afterAll } from 'vitest'
import { connectParachains, connectVertical } from '@acala-network/chopsticks'
import { setTimeout } from 'timers/promises'

import * as SpiritnetConfig from '../network/spiritnet.js'
import * as PolkadotConfig from '../network/polkadot.js'
import * as HydraDxConfig from '../network/hydraDx.js'
import type { Config } from '../network/types.js'
import { hexAddress, setStorage } from './utils.js'
import { keysAlice, keysBob, keysCharlie } from '../utils.js'

export let spiritnetContext: Config
export let hydradxContext: Config
export let polkadotContext: Config

// There is not really a way to reset the storage. dev.setStorage only appends or overwrites an existing entry
beforeAll(async () => {
	spiritnetContext = await SpiritnetConfig.getContext()
	hydradxContext = await HydraDxConfig.getContext()
	polkadotContext = await PolkadotConfig.getContext()

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

	await setStorage(spiritnetContext, SpiritnetConfig.setSafeXcmVersion(3))
	await setStorage(hydradxContext, HydraDxConfig.registerKilt())
}, 60_000)

afterAll(async () => {
	// fixes api runtime disconnect warning
	await setTimeout(50)
	await Promise.all([spiritnetContext.teardown(), hydradxContext.teardown(), polkadotContext.teardown()])
})

// Resets the balance storage after each test
afterEach(async () => {
	console.log('Resetting balance storage')
	const accounts = [
		keysAlice.address,
		keysBob.address,
		keysCharlie.address,
		SpiritnetConfig.hydraDxSovereignAccount,
		HydraDxConfig.omnipoolAccount,
	]

	const hydraDxConfig = {
		...HydraDxConfig.assignNativeTokensToAccount(accounts, BigInt(0)),
		...HydraDxConfig.assignKiltTokensToAccount(accounts, BigInt(0)),
	}

	await setStorage(hydradxContext, hydraDxConfig)
	await setStorage(spiritnetContext, SpiritnetConfig.assignNativeTokensToAccount(accounts, BigInt(0)))
	await setStorage(polkadotContext, PolkadotConfig.setAddrNativeTokens(accounts, BigInt(0)))
})

export async function getFreeBalanceSpiritnet(account: string): Promise<bigint> {
	const accountInfo = await spiritnetContext.api.query.system.account(account)
	return accountInfo.data.free.toBigInt()
}

export async function getFreeBalanceHydraDxKilt(account: string): Promise<bigint> {
	const accountInfo: any = await hydradxContext.api.query.tokens.accounts(account, HydraDxConfig.kiltTokenId)
	return accountInfo.free.toBigInt()
}
