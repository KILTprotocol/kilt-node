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
}

export const defaultStorage = (addr: string) => ({
	// set technical committee and council to Bob
	technicalCommittee: { Members: [addr] },
	council: { Members: [addr] },
	System: {
		Account: [[[addr], { providers: 1, data: { free: 1000 * 1e12 } }]],
	},
})

export const spiritnet = {
	paraId: 2086,
	hydraDxDestination: {
		V3: {
			parents: 1,
			interior: {
				X1: {
					Parachain: HydraDxConfig.paraId,
				},
			},
		},
	},
	hydraDxBeneficiary: {
		V3: {
			parents: 1,
			interior: {
				X1: {
					AccountId32: {
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
