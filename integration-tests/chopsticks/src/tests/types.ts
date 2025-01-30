import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { SetupOption, EventFilter } from '@acala-network/chopsticks-testing'

export interface Storage {
	parachains: Record<string, Record<string, unknown>>[]
	relay: Record<string, Record<string, unknown>>
}

export interface Accounts {
	senderAccount: KeyringPair
	receiverAccount: KeyringPair
}

export interface NetworkSetupOption {
	parachains: SetupOption[]
	relay: SetupOption
}

export interface BasicTxContext {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	tx: ({ api }: { api: ApiPromise }, submitter: string, ...args: any[]) => SubmittableExtrinsic<'promise'>
	pallets: {
		sender: EventFilter[]
		receiver: EventFilter[]
	}
}

export type SetupChain = 'relay' | 'sender' | 'receiver'

export interface BasicConfig {
	desc: string
	storage: Storage
	network: NetworkSetupOption
	setUpTx?: [({ api }: { api: ApiPromise }) => SubmittableExtrinsic<'promise'>, SetupChain][]
}

export interface SovereignAccount {
	sender: string
	receiver: string
}

export interface BasicXcmTestConfiguration {
	config: BasicConfig
	accounts: Accounts
	sovereignAccount: SovereignAccount
	txContext: BasicTxContext
}
