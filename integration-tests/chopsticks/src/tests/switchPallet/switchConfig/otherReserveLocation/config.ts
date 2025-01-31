import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { keysAlice, KILT } from '../../../../helper/utils.js'
import { mainChains } from '../../../../network/index.js'
import { tx, query } from '../../../../helper/api.js'
import type { BasicConfig } from '../../../types.js'
import { getDepositXcmMessageV3 } from '../../index.js'

interface Query {
	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface Events {
	// events to check after the transaction on the native chain
	sender: EventFilter[]
	// events to check after the transaction on the foreign chain
	receiver: EventFilter[]
}

interface TxContext {
	// amount of funds to transfer
	balanceToTransfer: bigint
	// transactions to execute
	tx: ({ api }: { api: ApiPromise }, xcmMessage: object) => SubmittableExtrinsic<'promise'>
	// the xcm message to send
	message: (amount: string, receiver: string) => object
	// events to check after the transaction
	events: Events
}

interface TestConfiguration {
	config: BasicConfig
	query: Query
	txContext: TxContext
	account: KeyringPair
}

// Test pairs for limited reserve transfers
export const testCases: TestConfiguration[] = [
	{
		config: {
			desc: 'V4 LIVE',
			network: {
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
				parachains: [{ option: mainChains.kilt.getConfig({}), setUpTx: [], storage: {} }],
			},
		},
		account: keysAlice,
		query: {
			receiver: query.balances,
		},
		txContext: {
			balanceToTransfer: KILT,
			message: getDepositXcmMessageV3(mainChains.assetHub.chainInfo.eKiltLocation),
			tx: tx.xcmPallet.send(tx.xcmPallet.parachainV3(0, mainChains.kilt.chainInfo.paraId)),
			events: {
				sender: [
					{
						section: 'xcmPallet',
						method: 'Sent',
					},
				],
				receiver: [
					{
						section: 'messageQueue',
						method: 'ProcessingFailed',
					},
				],
			},
		},
	},
]
