import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import * as SpiritnetConfig from './spiritnet.js'
import { initialBalanceHDX, initialBalanceKILT, toNumber } from '../utils.js'

export const options: SetupOption = {
	endpoint: process.env.HYDRADX_WS || ['wss://hydradx-rpc.dwellir.com', 'wss://rpc.hydradx.cloud'],
	db: './db/hydradx.db.sqlite',
	port: toNumber(process.env.HYDRADX_PORT) || 9001,
}

export const kiltTokenId = 60

export function setGovernance(addr: string[]) {
	return {
		TechnicalCommittee: { Members: addr },
		Council: { Members: addr },
	}
}

export function assignNativeTokensToAccount(addr: string, balance: bigint = initialBalanceHDX) {
	return {
		System: {
			Account: [[[addr], { providers: 1, data: { free: balance } }]],
		},
	}
}

export function assignKiltTokensToAccount(addr: string, balance: bigint = initialBalanceKILT) {
	return {
		Tokens: {
			Accounts: [[[addr, kiltTokenId], { free: balance }]],
		},
	}
}

export function registerKilt() {
	return {
		assetRegistry: {
			assetLocations: [[[kiltTokenId], { parents: 1, interior: { X1: { Parachain: SpiritnetConfig.paraId } } }]],
			assetIds: [[['KILT'], kiltTokenId]],
			locationAssets: [[[{ parents: 1, interior: { X1: { Parachain: SpiritnetConfig.paraId } } }], kiltTokenId]],
			assets: [
				[
					[kiltTokenId],
					{
						name: 'KILT',
						assetType: 'Token',
						existentialDeposit: 500,
						symbol: 'KILT',
						decimals: 18,
						xcmRateLimit: null,
						isSufficient: true,
					},
				],
			],
		},
		multiTransactionPayment: {
			acceptedCurrencies: [[[kiltTokenId], 100_000]],
		},
	}
}

export const paraId = 2034
export const omnipoolAccount = '7L53bUTBbfuj14UpdCNPwmgzzHSsrsTWBHX5pys32mVWM3C1'

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
