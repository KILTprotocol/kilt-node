import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { BasicConfig } from '../types.js'
import type { KeyringPair } from '@polkadot/keyring/types'

interface TxContext {
	// amount of funds to transfer
	balanceToTransfer: bigint
	// transactions to execute
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string) => SubmittableExtrinsic<'promise'>
}

interface TestConfiguration {
	config: BasicConfig
	txContext: TxContext
	account: KeyringPair
}

export const testCases: TestConfiguration[] = []
