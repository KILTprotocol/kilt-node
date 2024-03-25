import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

export const options: SetupOption = {
	endpoint: ['wss://rpc.hydradx.cloud', 'wss://hydradx-rpc.dwellir.com'],
	db: './db/hydradx.db.sqlite',
	port: 8001,
	runtimeLogLevel: 5,
}

enum Tokens {
	HDX = 0,
	LERNA = 1,
	// Kilt is not listed yet. Last token index is Interlay with 17.
	KILT = 18,
}

export const defaultStorage = (addr: string) => ({
	// set technical committee and council to addr
	TechnicalCommittee: { Members: [addr] },
	Council: { Members: [addr] },
	Tokens: {
		Accounts: [
			[[addr, Tokens.HDX], { free: 100 * 1e12 }],
			[[addr, Tokens.LERNA], { free: 100 * 1e12 }],
			[[addr, Tokens.KILT, { free: 100 * 1e12 }]],
		],
	},
})

export const paraId = 2034
export const sovereignAccount = u8aToHex(decodeAddress('5Eg2fntQqFi3EvFWAf71G66Ecjjah26bmFzoANAeHFgj9Lia'))

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
