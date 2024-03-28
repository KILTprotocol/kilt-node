import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import * as HydraDxConfig from './hydraDx.js'
import { initBalance, toNumber } from './utils.js'

export const options: SetupOption = {
	endpoint: process.env.SPIRITNET_WS || 'wss://kilt-rpc.dwellir.com',
	db: './db/spiritnet.db.sqlite',
	port: toNumber(process.env.SPIRITNET_PORT) || 9002,
	wasmOverride: '../../target/debug/wbuild/spiritnet-runtime/spiritnet_runtime.wasm',
	// Whether to allow WASM unresolved imports when using a WASM to build the parachain
	allowUnresolvedImports: true,
}

export const defaultStorage = (addr: string) => ({
	technicalCommittee: { Members: [addr] },
	council: { Members: [addr] },
	System: {
		Account: [[[addr], { providers: 1, data: { free: initBalance } }]],
	},
	polkadotXcm: {
		safeXcmVersion: 3,
	},
})

export const paraId = 2086

const hydraDxLocation = {
	parents: 1,
	interior: {
		X1: {
			Parachain: HydraDxConfig.paraId,
		},
	},
}

const nativeAssetIdLocation = (amount: number) => [
	{
		id: { Concrete: { parents: 0, interior: 'Here' } },
		fun: { Fungible: amount },
	},
]

export const V2 = {
	hydraDxDestination: {
		V2: hydraDxLocation,
	},
	hydraDxBeneficiary: {
		V2: {
			parents: 0,
			interior: {
				X1: {
					AccountId32: {
						network: 'Any',
						id: HydraDxConfig.omnipoolAccount,
					},
				},
			},
		},
	},

	nativeAssetId: (amount: number) => ({
		V2: nativeAssetIdLocation(amount),
	}),
}

export const V3 = {
	hydraDxDestination: {
		V3: hydraDxLocation,
	},

	hydraDxBeneficiary: {
		V3: {
			parents: 0,
			interior: {
				X1: {
					AccountId32: {
						id: HydraDxConfig.omnipoolAccount,
					},
				},
			},
		},
	},

	nativeAssetId: (amount: number) => ({
		V3: nativeAssetIdLocation(amount),
	}),
}

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
