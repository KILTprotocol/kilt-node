/* eslint-disable @typescript-eslint/no-explicit-any */
import * as PolkadotChainConfigs from '../../../network/index.js'
import {
	hexAddress,
	initialBalanceDOT,
	initialBalanceKILT,
	keysAlice,
	keysBob,
	keysCharlie,
} from '../../../helper/utils.js'
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
	balanceToTransfer: bigint
	tx: ({ api }: { api: ApiPromise }, beneficiary: any, amount: string | number) => SubmittableExtrinsic<'promise'>
	destination: any
}

/*
 * Configuration for the SwapPairConfiguration test extends the BasicXcmTestConfiguration
 **/
interface SwapPairTestConfiguration extends BasicXcmTestConfiguration {
	config: Config
	query: Query
	txContext: TxContext
}

// Test pairs for swapping assets
export const testPairsSwapAssets: SwapPairTestConfiguration[] = [
	{
		config: {
			desc: 'KILT -> AssetHub',
			precision: BigInt(99),
		},
		network: {
			sender: PolkadotChainConfigs.all.spiritnet.getConfig({
				wasmOverride: PolkadotChainConfigs.all.spiritnet.parameters.wasmOverride,
			}),
			receiver: PolkadotChainConfigs.all.assetHub.getConfig({}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysBob,
		},
		query: {
			sender: query.balances,
			receiver: query.balances,
		},
		txContext: {
			tx: tx.assetSwap.swap(),
			pallets: {
				sender: [],
				receiver: [],
			},
			destination: tx.assetSwap.beneficiaryV3(hexAddress(keysBob.address)),
			balanceToTransfer: BigInt(1e15),
		},
		storage: {
			senderStorage: {
				...SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
			},
			receiverStorage: {
				...AssetHubConfig.assignDotTokensToAccounts([keysAlice.address], initialBalanceDOT),
				...AssetHubConfig.createForeignAsset(keysCharlie.address),
				...AssetHubConfig.assignForeignAssetToAccounts([SpiritnetConfig.siblingSovereignAccount]),
			},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: AssetHubConfig.siblingSovereignAccount,
			receiver: SpiritnetConfig.siblingSovereignAccount,
		},
	},
]
