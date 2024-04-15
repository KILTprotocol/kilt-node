import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'
import type { Config } from './types.js'
import { initialBalanceKILT, toNumber } from '../utils.js'

/// Options used to create the Spiritnet context
const options: SetupOption = {
	endpoint: process.env.SPIRITNET_WS || 'wss://kilt-rpc.dwellir.com',
	db: './db/spiritnet.db.sqlite',
	port: toNumber(process.env.SPIRITNET_PORT) || 9002,
	wasmOverride: '../../target/debug/wbuild/spiritnet-runtime/spiritnet_runtime.wasm',
	// Whether to allow WASM unresolved imports when using a WASM to build the parachain. This Flag is needed otherwise, the runtime can not be built from the WASM. Chopsticks throws an error when it encounters an unresolved import.
	allowUnresolvedImports: true,
}

/// Assigns the native tokens to an account
export function assignNativeTokensToAccount(addr: string[], balance: bigint = initialBalanceKILT) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
		},
	}
}

/// Sets the [technicalCommittee] and [council] governance to the given accounts
export function setGovernance(addr: string[]) {
	return {
		technicalCommittee: { Members: addr },
		council: { Members: addr },
	}
}

/// Sets the [safeXcmVersion] to the given version
export function setSafeXcmVersion(version: number) {
	return {
		polkadotXcm: {
			safeXcmVersion: version,
		},
	}
}

/// Spiritnet ParaId
export const paraId = 2086

/// The sovereign account of HydraDx in Spiritnet
export const hydraDxSovereignAccount = '4qXPdpioJ6D8cgdeYXaukV2Y2oAQUHaX1VnGhdbSRqJn2CBt'

/// Returns the Spiritnet context for the given options
export async function getContext(): Promise<Config> {
	return setupContext(options)
}
