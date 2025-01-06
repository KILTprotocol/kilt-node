import * as PolkadotChainConfigs from '../../../network/index.js'
import { hexAddress, initialBalanceKILT, keysAlice, keysBob } from '../../../helper/utils.js'
import * as SpiritnetConfig from '../../../network/spiritnet.js'
import * as AssetHubContext from '../../../network/assethub.js'
import { tx, query } from '../../../helper/api.js'

import type { ApiPromise } from '@polkadot/api'
import type { Accounts, BasicConfig, NetworkSetupOption, SovereignAccount, Storage } from '../../types.js'
import type { SubmittableExtrinsic } from '@polkadot/api/types'

interface QueryFunds {
	native: ({ api }: { api: ApiPromise }, address: string) => Promise<bigint>
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	foreign: () => void //({ api }: { api: ApiPromise }, assetId: any, address: string) => Promise<bigint>
}

interface Query {
	sender: QueryFunds
	receiver: QueryFunds
}

interface TxContext {
	balanceToTransfer: {
		native: bigint
		foreign: bigint
	}
	tx: {
		switch: ({ api }: { api: ApiPromise }, submitter: string, amount: string) => SubmittableExtrinsic<'promise'>
		switchBack: ({ api }: { api: ApiPromise }, amount: string) => SubmittableExtrinsic<'promise'>
		transfer: (
			{ api }: { api: ApiPromise },
			beneficiary1: string,
			amount: string | number
		) => SubmittableExtrinsic<'promise'>
		withdraw: (
			{ api }: { api: ApiPromise },
			submitter: string,
			amount: string | number
		) => SubmittableExtrinsic<'promise'>
	}
}

/*
 * Configuration for Swtichting coins.
 **/
interface SwitchTestConfiguration {
	config: BasicConfig
	query: Query
	txContext: TxContext
	network: NetworkSetupOption
	accounts: Accounts
	storage: Storage
	sovereignAccount: SovereignAccount
}

export function getXcmMessageV4ToSendEkilt(address: string) {
	return {
		V4: [
			{
				DepositAsset: {
					assets: { Wild: 'All' },
					beneficiary: {
						parents: 0,
						interior: {
							X1: [
								{
									AccountId32: {
										id: hexAddress(address),
									},
								},
							],
						},
					},
				},
			},
		],
	}
}

// Test pairs for limited reserve transfers
export const testPairsSwitchFunds: SwitchTestConfiguration[] = [
	{
		config: {
			desc: 'Switch: Kilt -> AssetHub -> Kilt',
		},
		network: {
			sender: PolkadotChainConfigs.all.spiritnet.getConfig({}),
			receiver: PolkadotChainConfigs.all.assetHub.getConfig({}),
			relay: PolkadotChainConfigs.all.polkadot.getConfig({}),
		},
		accounts: {
			senderAccount: keysAlice,
			receiverAccount: keysAlice,
		},
		query: {
			sender: { native: query.balances, foreign: () => {} },
			receiver: { native: query.balances, foreign: () => {} },
		},
		txContext: {
			tx: {
				switch: tx.switchPallet.switchV4(),
				switchBack: tx.xcmPallet.transferAssetsUsingTypeAndThen(
					tx.xcmPallet.parachainV4(1, SpiritnetConfig.paraId),
					AssetHubContext.eKiltLocation,
					getXcmMessageV4ToSendEkilt(keysAlice.address)
				),
				transfer: tx.xcmPallet.limitedReserveTransferAssetsV3(
					{ Concrete: AssetHubContext.nativeTokenLocation },
					tx.xcmPallet.parachainV3(1, SpiritnetConfig.paraId)
				),
				withdraw: tx.xcmPallet.transferAssets(tx.xcmPallet.parachainV3(1, AssetHubContext.paraId), {
					Concrete: AssetHubContext.nativeTokenLocation,
				}),
			},
			// pallets: {
			// 	sender: [],
			// 	receiver: [],
			// },
			balanceToTransfer: {
				native: BigInt(1e15),
				foreign: BigInt(1e10),
			},
		},
		storage: {
			senderStorage: SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
			receiverStorage: {
				// Assign some coins to create the account.
				...AssetHubContext.assignDotTokensToAccountsAsStorage([keysAlice.address]),
				// Create the eKilts.
				...AssetHubContext.createForeignAsset(keysBob.address),
			},
			relayStorage: {},
		},
		sovereignAccount: {
			sender: AssetHubContext.sovereignAccountOnSiblingChains,
			receiver: SpiritnetConfig.siblingSovereignAccount,
		},
	},
] as const
