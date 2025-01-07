import { SetupOption } from '@acala-network/chopsticks-testing'

import { initialBalanceDOT, toNumber } from '../helper/utils.js'
import { ParachainInfo } from './types.js'

/// Options used to create the Spiritnet context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) =>
	({
		endpoint: process.env.ASSETHUB_WSS || 'wss://asset-hub-polkadot-rpc.dwellir.com',
		db: './db/assethub.db.sqlite',
		port: toNumber(process.env.ASSETHUB_PORT),
		wasmOverride,
		blockNumber,
	}) as SetupOption

export const storage = {
	/// AssetHub has no own coin. Teleported dots are used as the native token.
	assignDotTokensToAccountsAsStorage(addr: string[], balance: bigint = initialBalanceDOT) {
		return {
			System: {
				Account: addr.map((address) => [[address], { providers: 1, data: { free: balance.toString() } }]),
			},
		}
	},

	createForeignAsset(manager: string, assetId = parachainInfo.eKiltLocation) {
		return {
			foreignAssets: {
				asset: [
					[
						[assetId],
						{
							owner: manager,
							issuer: manager,
							admin: manager,
							freezer: manager,
							// Just make it big enough
							supply: '10000000000000000000000000000',
							deposit: 0,
							minBalance: 0,
							isSufficient: false,
							accounts: 0,
							sufficients: 0,
							approvals: 0,
							status: 'Live',
						},
					],
				],
			},
		}
	},

	/// Assigns KSM to an account
	assignKSMtoAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
		return {
			foreignAssets: {
				account: addr.map((addr) => [
					[parachainInfo.KSMAssetLocation, addr],
					{
						balance: balance,
						status: 'Liquid',
						reason: 'Consumer',
						extra: null,
					},
				]),
			},
		}
	},

	/// Assigns the foreign asset to the accounts.
	/// Does not check if supply is matching the sum of the account balances.
	assignForeignAssetToAccounts(accountInfo: [string, bigint][], assetId = parachainInfo.eKiltLocation) {
		return {
			foreignAssets: {
				account: accountInfo.map(([account, balance]) => [
					[assetId, account],
					{
						balance: balance,
						status: 'Liquid',
						reason: 'Consumer',
						extra: null,
					},
				]),
			},
		}
	},
}

export const parachainInfo: ParachainInfo = {
	/// AssetHub ParaId
	paraId: 1000,
	KSMAssetLocation: {
		parents: 2,
		interior: {
			X1: {
				GlobalConsensus: 'Kusama',
			},
		},
	},

	// Sibling Sovereign Account
	sovereignAccountOnSiblingChains: '4qXPdpimHh8TR24RSk994yVzxx4TLfvKj5i1qH5puvWmfAqy',

	/// Native token in AssetHub
	nativeTokenLocation: { parents: 1, interior: 'Here' },

	eKiltLocation: {
		parents: 2,
		interior: {
			X2: [
				{
					GlobalConsensus: { Ethereum: { chainId: 1 } },
				},
				{
					AccountKey20: {
						network: null,
						key: '0x5d3d01fd6d2ad1169b17918eb4f153c6616288eb',
					},
				},
			],
		},
	},
}
