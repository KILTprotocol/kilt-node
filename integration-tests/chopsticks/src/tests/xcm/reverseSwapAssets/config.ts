/* eslint-disable @typescript-eslint/no-explicit-any */
import * as PolkadotChainConfigs from '../../../network/index.js'
import { initialBalanceDOT, keysAlice, keysCharlie } from '../../../helper/utils.js'
import * as SpiritnetConfig from '../../../network/spiritnet.js'
import * as AssetHubConfig from '../../../network/assethub.js'
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
	tx: ({ api }: { api: ApiPromise }, beneficiary: any, funds: any) => SubmittableExtrinsic<'promise'>
}

/*
 * Configuration for the SwapPairConfiguration test extends the BasicXcmTestConfiguration
 **/
interface ReverseSwapPairTestConfiguration extends BasicXcmTestConfiguration {
	config: Config
	query: Query
	txContext: TxContext
}

// Test pairs for swapping assets
export const testPairsSwapAssets: ReverseSwapPairTestConfiguration[] = [
	{
		config: {
			desc: 'AssetHub -> KILT',
			precision: BigInt(99),
		},
		network: {
			sender: PolkadotChainConfigs.all.assetHub.getConfig({}),
			receiver: PolkadotChainConfigs.all.spiritnet.getConfig({
				wasmOverride: PolkadotChainConfigs.all.spiritnet.parameters.wasmOverride,
			}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysAlice,
		},
		query: {
			sender: query.balances,
			receiver: query.balances,
		},
		txContext: {
			tx: tx.assetSwap.transferAssetsUsingTypeAndThen(tx.xcmPallet.parachainV3(1, SpiritnetConfig.paraId), {
				V3: { Concrete: tx.assetSwap.asset },
			}),
			pallets: {
				sender: [],
				receiver: [],
			},
		},
		storage: {
			senderStorage: {
				...AssetHubConfig.assignDotTokensToAccounts(
					[
						keysAlice.address,
						SpiritnetConfig.siblingSovereignAccount,
						'5DPiZzQQdoJJucxGMCgrJEdeUkLfPs6fndeCMA1E4ZgAkWyh',
					],
					initialBalanceDOT
				),
				...AssetHubConfig.createForeignAsset(keysCharlie.address, [
					SpiritnetConfig.siblingSovereignAccount,
					keysAlice.address,
					'5DPiZzQQdoJJucxGMCgrJEdeUkLfPs6fndeCMA1E4ZgAkWyh',
				]),
			},
			receiverStorage: {
				...SpiritnetConfig.createAndAssignDots(keysCharlie.address, [keysAlice.address]),
				...SpiritnetConfig.setSwapPair(),
				...SpiritnetConfig.setSafeXcmVersion3(),
			},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: SpiritnetConfig.siblingSovereignAccount,
			receiver: AssetHubConfig.siblingSovereignAccount,
		},
	},
]
