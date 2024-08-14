import { setupContext, SetupOption } from '@acala-network/chopsticks-testing'

import type { Config } from './types.js'
import { initialBalanceDOT, toNumber } from '../utils.js'

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
		port: toNumber(process.env.ASSETHUB_PORT) || 9003,
		wasmOverride,
		blockNumber,
	}) as SetupOption

/// AssetHub has no own coin. Teleported dots are used as the native token.
export function assignDotTokensToAccountsAsStorage(addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance.toString() } }]),
		},
	}
}

export function createForeignAsset(manager: string, assetId = eKiltLocation) {
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
}

/// Assigns KSM to an account
export function assignKSMtoAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		foreignAssets: {
			account: addr.map((addr) => [
				[KSMAssetLocation, addr],
				{
					balance: balance,
					status: 'Liquid',
					reason: 'Consumer',
					extra: null,
				},
			]),
		},
	}
}

/// Assigns the foreign asset to the accounts.
/// Does not check if supply is matching the sum of the account balances.
export function assignForeignAssetToAccounts(accountInfo: [string, bigint][], assetId = eKiltLocation) {
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
}

/// AssetHub ParaId
export const paraId = 1000

export const KSMAssetLocation = {
	parents: 2,
	interior: {
		X1: {
			GlobalConsensus: 'Kusama',
		},
	},
}

// Sibling Sovereign Account
export const sovereignAccountOnSiblingChains = '4qXPdpimHh8TR24RSk994yVzxx4TLfvKj5i1qH5puvWmfAqy'

/// Native token in AssetHub
export const nativeTokenLocation = { parents: 1, interior: 'Here' }

export const eKiltLocation = {
	parents: 2,
	interior: {
		X2: [
			{
				GlobalConsensus: { Ethereum: { chainId: 11155111 } },
			},
			{
				AccountKey20: {
					network: null,
					key: '0x06012c8cf97bead5deae237070f9587f8e7a266d',
				},
			},
		],
	},
}

export async function getContext(): Promise<Config> {
	const options = getSetupOptions({})
	return setupContext(options)
}
