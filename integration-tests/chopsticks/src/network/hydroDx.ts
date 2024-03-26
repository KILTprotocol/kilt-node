import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

export const options: SetupOption = {
	endpoint: ['wss://hydradx-rpc.dwellir.com', 'wss://rpc.hydradx.cloud'],
	db: './db/hydradx.db.sqlite',
	port: 8001,
}

enum Tokens {
	HDX = 0,
	LERNA = 1,
	KILT = 60,
}

export const defaultStorage = (addr: string) => ({
	// set technical committee and council to addr
	TechnicalCommittee: { Members: [addr] },
	Council: { Members: [addr] },
	Tokens: {
		Accounts: [
			[['4pF5Y2Eo6doQHPLQj5AkndZwtomVB8ab2sRftRS2D9JDdELr', Tokens.KILT], { free: 1000 * 1e12 }],
			[['4pF5Y2Eo6doQHPLQj5AkndZwtomVB8ab2sRftRS2D9JDdELr', Tokens.HDX], { free: 1000 * 1e12 }],
		],
	},
	assetRegistry: {
		assetLocations: [[[Tokens.KILT], { parents: 1, interior: { X1: { Parachain: 2086 } } }]],
		assetIds: [[['KILT'], Tokens.KILT]],
		locationAssets: [[[{ parents: 1, interior: { X1: { Parachain: 2086 } } }], Tokens.KILT]],
		assets: [
			[
				[Tokens.KILT],
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
		acceptedCurrencies: [[[Tokens.KILT], 100_000]],
	},

	System: {
		Account: [
			[['5Eg2fnshxV9kofpcNEFE7azHLAjcCtpNkbsH3kkWZasYUVKs'], { providers: 1, data: { free: 1000 * 1e12 } }],
		],
	},
})

export const paraId = 2034
export const sovereignAccount = u8aToHex(decodeAddress('5Eg2fntQqFi3EvFWAf71G66Ecjjah26bmFzoANAeHFgj9Lia'))

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
