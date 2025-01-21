import type { SetupOption, setupContext } from '@acala-network/chopsticks-testing'

export type Config = Awaited<ReturnType<typeof setupContext>>

export interface Chain {
	getConfig: ({ blockNumber, wasmOverride }: { blockNumber?: number; wasmOverride?: string }) => SetupOption

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	storage: Record<string, (...args: any[]) => Record<string, Record<string, unknown>>>
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	chainInfo: Record<string, any>
}

export interface Chains {
	test?: Chain
	main?: Chain
}

export interface ChainConfig extends Chain {
	parameters: {
		blockNumber?: number
		wasmOverride?: string
	}
}

type chains = 'kilt' | 'polkadot' | 'assetHub' | 'hydration'
/// A Record of all possible chain configurations.
export type ChainConfigs = Record<chains, ChainConfig>
