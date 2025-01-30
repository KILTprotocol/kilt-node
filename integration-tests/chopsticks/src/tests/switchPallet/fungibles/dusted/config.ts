import type { EventFilter } from '@acala-network/chopsticks-testing'
import type { ApiPromise } from '@polkadot/api'
import type { SubmittableExtrinsic } from '@polkadot/api/types'
import type { KeyringPair } from '@polkadot/keyring/types'

import { initialBalanceDOT, initialBalanceKILT, keysAlice, keysBob } from '../../../../helper/utils.js'
import { mainChains } from '../../../../network/index.js'
import { tx, query } from '../../../../helper/api.js'
import type { BasicConfig } from '../../../types.js'

interface Query {
	native: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	foreign: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface TxContext {
	balanceToTransfer: bigint
	tx: ({ api }: { api: ApiPromise }, destination: string, amount: string) => SubmittableExtrinsic<'promise'>
	events: EventFilter[]
}

interface TestConfiguration {
	config: BasicConfig
	query: Query
	txContext: TxContext
	account: {
		sender: KeyringPair
		receiver: KeyringPair
	}
}

export const testCases: TestConfiguration[] = [
	{
		config: {
			desc: 'V4 LIVE',
			network: {
				relay: mainChains.polkadot.getConfig({}),
				parachains: [mainChains.kilt.getConfig({})],
			},
			storage: {
				relay: {},
				parachains: [
					// sender
					{
						...mainChains.kilt.storage.assignNativeTokensToAccounts(
							[keysAlice.address],
							initialBalanceKILT
						),
						...mainChains.kilt.storage.assignRelayTokensToAccounts([keysAlice.address], initialBalanceDOT),
					},
					// receiver
					{},
				],
			},
		},
		account: { sender: keysAlice, receiver: keysBob },
		query: {
			native: query.balances,
			foreign: query.fungibles(mainChains.assetHub.chainInfo.nativeTokenLocation),
		},
		txContext: {
			tx: tx.balances.transferAllowDeath,
			events: [
				{ section: 'balances', method: 'Transfer' },
				{ section: 'balances', method: 'Endowed' },
			],
			balanceToTransfer: initialBalanceKILT,
		},
	},
] as const
