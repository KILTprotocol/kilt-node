import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { SetupOption, EventFilter } from '@acala-network/chopsticks-testing'

export interface Storage {
	senderStorage: Record<string, Record<string, unknown>>
	receiverStorage: Record<string, Record<string, unknown>>
	relayStorage: Record<string, Record<string, unknown>>
}

export interface Accounts {
	senderAccount: KeyringPair
	receiverAccount: KeyringPair
}

export interface NetworkSetupOption {
	sender: SetupOption
	receiver: SetupOption
	relay: SetupOption
}

export interface BasisTxContext {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	tx: ({ api }: { api: ApiPromise }, submitter: string, ...args: any[]) => SubmittableExtrinsic<'promise'>
	pallets: {
		sender: EventFilter[]
		receiver: EventFilter[]
	}
}

export interface BasicConfig {
	desc: string
}

interface SovereignAccount {
	sender: string
	receiver: string
}

export interface BasicXcmTestConfiguration {
	config: BasicConfig
	storage: Storage
	accounts: Accounts
	network: NetworkSetupOption
	sovereignAccount: SovereignAccount
	txContext: BasisTxContext
}
