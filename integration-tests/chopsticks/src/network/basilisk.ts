import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import { initialBalanceDOT, initialBalanceHDX, initialBalanceKILT, toNumber } from '../utils.js'

/// Options used to create the HydraDx context
export const options: SetupOption = {
	endpoint: process.env.BASILISK_WS || ['wss://basilisk-rococo-rpc.play.hydration.cloud'],
	db: './db/basilisk.db.sqlite',
	port: toNumber(process.env.HYDRADX_PORT) || 9005,
}

// On Basilisk, there is only KSM. We use that currency and treat it as ROC, since the location is {parents: 1, interior: Here}
export const dotTokenId = 5

export const eKILTTokenId = 90

/// Assigns the native tokens to an accounts
export function assignNativeTokensToAccounts(addr: string[], balance: bigint = initialBalanceHDX) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
		},
	}
}

/// Assigns ROCs tokens to an accounts
export function assignRocTokensToAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		Tokens: {
			Accounts: addr.map((address) => [[address, dotTokenId], { free: balance }]),
		},
	}
}

export function createRemoteAsset(addr: string[], assetId: Object, balance: bigint = initialBalanceKILT) {
	return {
		assetRegistry: {
			assetIds: [[['eKILT'], 90]],
			assetLocations: [[[90], assetId]],
			assetMetadataMap: [[[90], { symbol: 'eKILT', decimals: 15 }]],
		},

		tokens: {
			accounts: addr.map((acc) => [[acc, 90], { free: balance }]),
		},
	}
}

/// Basilisk ParaId
export const paraId = 2090

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
