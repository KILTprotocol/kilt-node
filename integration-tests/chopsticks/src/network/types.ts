import type { SetupOption, setupContext } from '@acala-network/chopsticks-testing'

export type Config = Awaited<ReturnType<typeof setupContext>>

export interface Chain {
	config: (blockNumber?: number, wasmOverride?: string) => SetupOption
	blockNumber?: number
	wasmOverride?: string
	name: string
}

export type ChainConfigs = Record<string, Chain>
