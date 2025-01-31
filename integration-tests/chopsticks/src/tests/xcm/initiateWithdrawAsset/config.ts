import { initialBalanceHDX, initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import { tx, query } from '../../../helper/api.js'
import { mainChains } from '../../../network/index.js'

import type { ApiPromise } from '@polkadot/api'
import type { BasicConfig, BasicXcmTestConfiguration, BasicTxContext } from '../../types.js'
import type { SubmittableExtrinsic } from '@polkadot/api/types'

interface Config extends BasicConfig {
	// The received balance can be different in each block due to dynamic fee calculation.
	// Precision is used to compare the balances with a certain precision.
	precision: bigint
}

interface Query {
	sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface TxContext extends BasicTxContext {
	balanceToTransfer: bigint
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string | number) => SubmittableExtrinsic<'promise'>
}

/*
 * Configuration for the WithdrawAssets test extends the BasicXcmTestConfiguration
 **/
interface WithdrawAssetTestConfiguration extends BasicXcmTestConfiguration {
	config: Config
	query: Query
	txContext: TxContext
}
// Test pairs for WithdrawAssets
export const testPairsWithdrawAssets: WithdrawAssetTestConfiguration[] = [
	{
		config: {
			desc: 'Hydration -> KILT live',
			precision: BigInt(96),
			network: {
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
				// sender, receiver
				parachains: [
					{
						option: mainChains.hydration.getConfig({}),
						storage: {
							...mainChains.hydration.storage.assignKiltTokensToAccounts(
								[keysAlice.address],
								initialBalanceKILT
							),
							...mainChains.hydration.storage.assignNativeTokensToAccounts(
								[keysAlice.address],
								initialBalanceHDX
							),
						},
						setUpTx: [],
					},

					{
						option: mainChains.kilt.getConfig({}),
						storage: {},
						setUpTx: [],
					},
				],
			},
		},

		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.tokens(mainChains.hydration.chainInfo.kiltTokenId),
			receiver: query.balances,
		},
		txContext: {
			tx: tx.xtokens.transfer(
				mainChains.hydration.chainInfo.kiltTokenId,
				tx.xtokens.parachainV3(mainChains.kilt.chainInfo.paraId)
			),
			pallets: {
				sender: [],
				receiver: [{ section: 'system', method: 'NewAccount' }],
			},
			balanceToTransfer: BigInt(1e15),
		},

		sovereignAccount: {
			sender: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.hydration.chainInfo.sovereignAccountOnSiblingChains,
		},
	},
	{
		config: {
			desc: 'Hydration -> KILT DEV',
			precision: BigInt(96),
			network: {
				relay: { option: mainChains.polkadot.getConfig({}), storage: {}, setUpTx: [] },
				// sender, receiver
				parachains: [
					{
						option: mainChains.hydration.getConfig({}),
						storage: {
							...mainChains.hydration.storage.assignKiltTokensToAccounts(
								[keysAlice.address],
								initialBalanceKILT
							),
							...mainChains.hydration.storage.assignNativeTokensToAccounts(
								[keysAlice.address],
								initialBalanceHDX
							),
						},
						setUpTx: [],
					},

					{
						option: mainChains.kilt.getConfig(mainChains.kilt.parameters),
						storage: {},
						setUpTx: [],
					},
				],
			},
		},

		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.tokens(mainChains.hydration.chainInfo.kiltTokenId),
			receiver: query.balances,
		},
		txContext: {
			tx: tx.xtokens.transfer(
				mainChains.hydration.chainInfo.kiltTokenId,
				tx.xtokens.parachainV3(mainChains.kilt.chainInfo.paraId)
			),
			pallets: {
				sender: [],
				receiver: [{ section: 'system', method: 'NewAccount' }],
			},
			balanceToTransfer: BigInt(1e15),
		},

		sovereignAccount: {
			sender: mainChains.kilt.chainInfo.sovereignAccountOnSiblingChains,
			receiver: mainChains.hydration.chainInfo.sovereignAccountOnSiblingChains,
		},
	},
] as const
