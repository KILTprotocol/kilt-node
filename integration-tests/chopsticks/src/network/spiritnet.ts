import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import * as HydraDxConfig from './hydraDx.js'
import { toNumber } from './utils.js'

export const options: SetupOption = {
	endpoint: process.env.SPIRITNET_WS || 'wss://kilt-rpc.dwellir.com',
	db: './db/spiritnet.db.sqlite',
	port: toNumber(process.env.SPIRITNET_PORT) || 8002,
	wasmOverride: '../../target/debug/wbuild/spiritnet-runtime/spiritnet_runtime.wasm',
	// Whether to allow WASM unresolved imports when using a WASM to build the parachain
	allowUnresolvedImports: true,
}

export const defaultStorage = (addr: string) => ({
	technicalCommittee: { Members: [addr] },
	council: { Members: [addr] },
	System: {
		Account: [[[addr], { providers: 1, data: { free: 1000 * 1e12 } }]],
	},
	polkadotXcm: {
		safeXcmVersion: 3,
	},
})

export const paraId = 2086
export const hydraDxDestinationV2 = {
	V2: {
		parents: 1,
		interior: {
			X1: {
				Parachain: HydraDxConfig.paraId,
			},
		},
	},
}
export const hydraDxBeneficiaryV2 = {
	V2: {
		parents: 1,
		interior: {
			X1: {
				AccountId32: {
					network: 'Any',
					id: HydraDxConfig.sovereignAccount,
				},
			},
		},
	},
}

export const nativeAssetIdV2 = {
	V2: [
		{
			id: { Concrete: { parents: 0, interior: 'Here' } },
			fun: { Fungible: 10e12 },
		},
	],
}

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
