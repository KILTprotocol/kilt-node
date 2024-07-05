import { beforeEach, afterEach } from 'vitest'
import { connectParachains, connectVertical } from '@acala-network/chopsticks'
import { setTimeout } from 'timers/promises'

import * as SpiritnetConfig from '../network/spiritnet.js'
import * as PolkadotConfig from '../network/polkadot.js'
import * as HydraDxConfig from '../network/hydraDx.js'
import * as AssetHubConfig from '../network/assethub.js'
import * as RococoConfig from '../network/rococo.js'
import * as BasiliskConfig from '../network/basilisk.js'
import * as PeregrineConfig from '../network/peregrine.js'
import type { Config } from '../network/types.js'

export let spiritnetContext: Config
export let hydradxContext: Config
export let polkadotContext: Config
export let assethubContext: Config
export let peregrineContext: Config
export let rococoContext: Config
export let basiliskContext: Config

beforeEach(async () => {
	spiritnetContext = await SpiritnetConfig.getContext()
	hydradxContext = await HydraDxConfig.getContext()
	polkadotContext = await PolkadotConfig.getContext()
	assethubContext = await AssetHubConfig.getContext()
	rococoContext = await RococoConfig.getContext()
	peregrineContext = await PeregrineConfig.getContext()
	basiliskContext = await BasiliskConfig.getContext()

	// Setup Polkadot network
	await connectVertical(polkadotContext.chain, spiritnetContext.chain)
	await connectVertical(polkadotContext.chain, hydradxContext.chain)
	await connectParachains([spiritnetContext.chain, hydradxContext.chain])

	// setup Rococo Network
	await connectVertical(rococoContext.chain, assethubContext.chain)
	await connectVertical(rococoContext.chain, peregrineContext.chain)
	await connectVertical(rococoContext.chain, basiliskContext.chain)
	await connectParachains([peregrineContext.chain, basiliskContext.chain, assethubContext.chain])

	const newBlockConfig = { count: 2 }
	// fixes api runtime disconnect warning
	await setTimeout(50)
	// Perform runtime upgrade and establish xcm connections.
	await Promise.all([
		polkadotContext.dev.newBlock(newBlockConfig),
		spiritnetContext.dev.newBlock(newBlockConfig),
		hydradxContext.dev.newBlock(newBlockConfig),
		assethubContext.dev.newBlock(newBlockConfig),
		rococoContext.dev.newBlock(newBlockConfig),
		peregrineContext.dev.newBlock(newBlockConfig),
		basiliskContext.dev.newBlock(newBlockConfig),
	])
}, 30_000)

afterEach(async () => {
	// fixes api runtime disconnect warning
	try {
		await Promise.all([
			spiritnetContext.teardown(),
			hydradxContext.teardown(),
			polkadotContext.teardown(),
			assethubContext.teardown(),
			rococoContext.teardown(),
			peregrineContext.teardown(),
			basiliskContext.teardown(),
		])
	} catch (error) {
		if (!(error instanceof TypeError)) {
			console.error(error)
		}
	}
	await setTimeout(50)
})

export async function getFreeBalanceSpiritnet(account: string): Promise<bigint> {
	const accountInfo = await spiritnetContext.api.query.system.account(account)
	return accountInfo.data.free.toBigInt()
}

export async function getFreeBalancePeregrine(account: string): Promise<bigint> {
	const accountInfo = await peregrineContext.api.query.system.account(account)
	return accountInfo.data.free.toBigInt()
}

export async function getFreeRocPeregrine(account: string): Promise<bigint> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const accountInfo: any = await peregrineContext.api.query.fungibles.account(AssetHubConfig.ROC, account)
	if (accountInfo.isNone) {
		return BigInt(0)
	}
	return accountInfo.unwrap().balance.toBigInt()
}

export async function getFreeRocAssetHub(account: string): Promise<bigint> {
	const accountInfo = await assethubContext.api.query.system.account(account)
	return accountInfo.data.free.toBigInt()
}

export async function getRemoteLockedSupply(): Promise<bigint> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const switchPairInfo: any = await peregrineContext.api.query.assetSwitchPool1.switchPair()

	if (switchPairInfo.isNone) {
		return BigInt(0)
	}

	return switchPairInfo.unwrap().remoteAssetBalance.toBigInt()
}

export async function getFreeEkiltAssetHub(account: string): Promise<bigint> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const accountInfo: any = await assethubContext.api.query.foreignAssets.account(
		AssetHubConfig.eKiltLocation,
		account
	)
	if (accountInfo.isNone) {
		return BigInt(0)
	}

	return accountInfo.unwrap().balance.toBigInt()
}

export async function getFreeBalanceHydraDxKilt(account: string): Promise<bigint> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const accountInfo: any = await hydradxContext.api.query.tokens.accounts(account, HydraDxConfig.kiltTokenId)
	return accountInfo.free.toBigInt()
}
