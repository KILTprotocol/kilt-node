import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'
import * as SpiritnetConfig from './spiritnet.js'

export const options: SetupOption = {
	endpoint: ['wss://hydradx-rpc.dwellir.com', 'wss://rpc.hydradx.cloud'],
	db: './db/hydradx.db.sqlite',
	port: 8001,
}

const kiltTokenId = 60

export const defaultStorage = (addr: string) => ({
	TechnicalCommittee: { Members: [addr] },
	Council: { Members: [addr] },
	Tokens: {
		Accounts: [[[addr, kiltTokenId], { free: 100 * 10e12 }]],
	},
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
					symbol: 'KLT',
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

	System: {
		Account: [[[addr], { providers: 1, data: { free: 100 * 10e12 } }]],
	},
})

export const paraId = 2034
export const sovereignAccount = u8aToHex(decodeAddress('5Eg2fntQqFi3EvFWAf71G66Ecjjah26bmFzoANAeHFgj9Lia'))

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
