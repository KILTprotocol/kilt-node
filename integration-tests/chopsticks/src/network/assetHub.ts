import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'

import type { Config } from './types.js'
import { initialBalanceDOT, toNumber } from '../utils.js'

/// Options used to create the Spiritnet context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) =>
	({
		endpoint: process.env.ASSETHUB_WSS || 'wss://asset-hub-polkadot-rpc.dwellir.com',
		db: './db/assethub.db.sqlite',
		port: toNumber(process.env.ASSETHUB_PORT) || 9003,
		wasmOverride,
		blockNumber,
	}) as SetupOption

export function assignKSMtoAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		foreignAssets: {
			account: addr.map((addr) => [
				[KSMAssetLocation, addr],
				{
					balance: balance,
					status: 'Liquid',
					reason: 'Consumer',
					extra: null,
				},
			]),
		},
	}
}

/// AssetHub has no own coin. Teleported dots are used as the native token.
export function assignDotTokensToAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance.toString() } }]),
		},
	}
}

/// AssetHub ParaId
export const paraId = 1000

export const KSMAssetLocation = {
	parents: 2,
	interior: {
		X1: {
			GlobalConsensus: 'Kusama',
		},
	},
}

// Sibling Sovereign Account
export const sovereignAccountOnSiblingChains = '4qXPdpimHh8TR24RSk994yVzxx4TLfvKj5i1qH5puvWmfAqy'

export async function getContext(): Promise<Config> {
	const options = getSetupOptions({})
	return setupContext(options)
}
