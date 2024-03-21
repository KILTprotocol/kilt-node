import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types'

export const options: SetupOption = {
	endpoint: 'wss://kilt-rpc.dwellir.com',
	db: './db/spiritnet.db.sqlite',
	port: 8002,
	wasmOverride: '../../target/release/wbuild/spiritnet-runtime/spiritnet_runtime.compact.compressed.wasm',
	allowUnresolvedImports: true,
	runtimeLogLevel: 5,
	timeout: 600000,
	resume: true,
}

export const defaultStorage = {
	// set technical committee and council to Bob
	technicalCommittee: { Members: ['4qpE21nvgo8AmyNMi32T7r4LWitN7fJaUox4PmGRUJvGqH7W'] },
	council: { Members: ['4qpE21nvgo8AmyNMi32T7r4LWitN7fJaUox4PmGRUJvGqH7W'] },
	System: {
		Account: [
			[['4qpE21nvgo8AmyNMi32T7r4LWitN7fJaUox4PmGRUJvGqH7W'], { providers: 1, data: { free: 1000 * 1e12 } }],
		],
	},
}

export const spiritnet = {
	paraId: 2086,
}

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
