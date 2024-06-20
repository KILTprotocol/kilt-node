/* eslint-disable @typescript-eslint/no-explicit-any */

import * as PolkadotChainConfigs from '../../../network/polkadot/index.js'
import { initialBalanceHDX, initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import * as SpiritnetConfig from '../../../network/polkadot/spiritnet.js'
import * as HydraDxConfig from '../../../network/polkadot/hydraDx.js'
import { tx, query } from '../../../helper/api.js'

import type { ApiPromise } from '@polkadot/api'
import type { BasicConfig, BasicXcmTestConfiguration, BasisTxContext } from '../types.js'
import type { SubmittableExtrinsic } from '@polkadot/api/types'

interface Config extends BasicConfig {
	precision: bigint
}

interface Query {
	sender: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	receiver: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
}

interface TxContext extends BasisTxContext {
	balanceToTransfer: bigint
	tx: ({ api }: { api: ApiPromise }, submitter: string, amount: string | number) => SubmittableExtrinsic<'promise'>
}

interface WithdrawAssetTestConfiguration extends BasicXcmTestConfiguration {
	config: Config
	query: Query
	txContext: TxContext
}
// Test pairs for limited reserve transfers
export const testPairsLimitedReserveTransfers: WithdrawAssetTestConfiguration[] = [
	// Kilt -> HydraDx
	{
		config: {
			desc: 'HydraDx -> KILT live',
			precision: BigInt(95),
		},

		network: {
			sender: PolkadotChainConfigs.all.hydraDx.getConfig({}),
			receiver: PolkadotChainConfigs.all.spiritnet.getConfig({}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.tokens(HydraDxConfig.kiltTokenId),
			receiver: query.balances,
		},
		txContext: {
			tx: tx.xtokens.transfer(HydraDxConfig.kiltTokenId, tx.xtokens.parachainV3(SpiritnetConfig.paraId)),
			pallets: {
				sender: [],
				receiver: [],
			},
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: {
				...HydraDxConfig.assignKiltTokensToAccounts([keysAlice.address], initialBalanceKILT),
				...HydraDxConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceHDX),
			},
			receiverStorage: {},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: SpiritnetConfig.hydraDxSovereignAccount,
			receiver: SpiritnetConfig.hydraDxSovereignAccount,
		},
	},
] as const
