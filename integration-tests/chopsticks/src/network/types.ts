import type { SetupOption, setupContext } from '@acala-network/chopsticks-testing'

export type Config = Awaited<ReturnType<typeof setupContext>>

export interface Chain {
	getConfig: ({ blockNumber, wasmOverride }: { blockNumber?: number; wasmOverride?: string }) => SetupOption
	parameters: {
		blockNumber?: number
		wasmOverride?: string
	}
}

export type ChainConfigs = Record<string, Chain>
