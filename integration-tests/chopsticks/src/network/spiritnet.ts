import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types'
import * as HydraDxConfig from './hydroDx'

export const options: SetupOption = {
	endpoint: 'wss://kilt-rpc.dwellir.com',
	db: './db/spiritnet.db.sqlite',
	port: 8002,
	wasmOverride: '../../target/debug/wbuild/spiritnet-runtime/spiritnet_runtime.wasm',
	allowUnresolvedImports: true,
	timeout: 600000,
	runtimeLogLevel: 5,
}

export const defaultStorage = (addr: string) => ({
	// set technical committee and council to addr
	technicalCommittee: { Members: [addr] },
	council: { Members: [addr] },
	System: {
		Account: [[[addr], { providers: 1, data: { free: 1000 * 1e12 } }]],
	},
	polkadotXcm: {
		safeXcmVersion: 3,
	},
})

export const spiritnet = {
	paraId: 2086,
	hydraDxDestination: {
		V2: {
			parents: 1,
			interior: {
				X1: {
					Parachain: HydraDxConfig.paraId,
				},
			},
		},
	},
	hydraDxBeneficiary: {
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
	},
}

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
