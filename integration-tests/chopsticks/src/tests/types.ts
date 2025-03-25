import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'
import type { SetupOption, EventFilter } from '@acala-network/chopsticks-testing'

export type Storage = Record<string, Record<string, unknown>>

export type SetUpTx = ({ api }: { api: ApiPromise }) => SubmittableExtrinsic<'promise'>

export interface ChainConfig {
	option: SetupOption
	storage: Storage
	setUpTx: SetUpTx[]
}

export interface Accounts {
	senderAccount: KeyringPair
	receiverAccount: KeyringPair
}
export interface NetworkSetupOption {
	parachains: ChainConfig[]
	relay: ChainConfig
}

export interface BasicTxContext {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	tx: ({ api }: { api: ApiPromise }, submitter: string, ...args: any[]) => SubmittableExtrinsic<'promise'>
	pallets: {
		sender: EventFilter[]
		receiver: EventFilter[]
	}
}

export interface BasicConfig {
	desc: string
	network: NetworkSetupOption
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
