import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { initialBalanceKILT, keysAlice } from '../../../../helper/utils.js'
import { mainChains } from '../../../../network/index.js'
import { tx } from '../../../../helper/api.js'
import type { BasicConfig } from '../../../types.js'

/**
 * Context for the transaction to switch funds between chains.
 */
interface TxContext {
	// amount of funds to transfer
	balanceToTransfer: bigint
	// transactions to execute
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string) => SubmittableExtrinsic<'promise'>
}

/*
 * Configuration for Swtichting coins.
 **/
interface SwitchTestConfiguration {
	config: BasicConfig
	txContext: TxContext
	account: KeyringPair
}

export const testCases: SwitchTestConfiguration[] = [
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
					...mainChains.kilt.storage.puaseSwitch(),
				},
				receiverStorage: {},
				relayStorage: {},
			},
		},
		account: keysAlice,
		txContext: {
			balanceToTransfer: BigInt('1000000000000000'),
			tx: tx.switchPallet.switchV4(),
		},
	},
]
