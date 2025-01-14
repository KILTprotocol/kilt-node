import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { initialBalanceKILT, keysAlice, KILT } from '../../../../helper/utils.js'
import { mainChains } from '../../../../network/index.js'
import { tx } from '../../../../helper/api.js'
import type { BasicConfig } from '../../../types.js'

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

export const testCases: TestConfiguration[] = [
	{
		config: {
			desc: 'V4 LIVE',
			network: {
				relay: mainChains.polkadot.getConfig({}),
				parachains: [mainChains.kilt.getConfig({}), mainChains.assetHub.getConfig({})],
			},
			storage: {
				senderStorage: {
					...mainChains.kilt.storage.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
					...mainChains.kilt.storage.pauseSwitch(),
				},
				receiverStorage: {},
				relayStorage: {},
			},
		},
		account: keysAlice,
		txContext: {
			balanceToTransfer: KILT,
			tx: tx.switchPallet.switchV4(),
		},
	},
]
