import { SetupOption } from '@acala-network/chopsticks-testing'

import { initialBalanceDOT, initialBalanceKILT, toNumber } from '../helper/utils.js'

/// Options used to create the Spiritnet context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) =>
	({
		endpoint: process.env.SPIRITNET_WS || 'wss://peregrine.kilt.io',
		db: './db/spiritnet.db.sqlite',
		port: toNumber(process.env.SPIRITNET_PORT) || 9002,
		runtimeLogLevel: 5,
		wasmOverride,
		blockNumber,
	}) as SetupOption

/// Assigns the native tokens to an accounts
export function assignNativeTokensToAccounts(addr: string[], balance: bigint = initialBalanceKILT) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
		},
	}
}

/// Sets the [technicalCommittee] and [council] governance to the given accounts
export function setGovernance(addr: string[]) {
	return {
		technicalCommittee: { Members: addr },
		council: { Members: addr },
	}
}

export function setSafeXcmVersion3() {
	return {
		polkadotXcm: {
			safeXcmVersion: 3,
		},
	}
}

export function createAndAssignDots(manager: string, addr: string[], balance: bigint = initialBalanceDOT) {
	return {
		fungibles: {
			asset: [
				[
					[
						{
							parents: 1,
							interior: 'Here',
						},
					],
					{
						owner: '4qPZ8fv6BjGoGKzfx5LtBFnEUp2b5Q5C1ErrjBNGmoFTLNHG',
						issuer: manager,
						admin: manager,
						freezer: manager,
						supply: balance * BigInt(addr.length),
						deposit: 0,
						minBalance: 1,
						isSufficient: false,
						accounts: addr.length,
						sufficients: 0,
						approvals: 0,
						status: 'Live',
					},
				],
			],
			account: addr.map((acc) => [
				[{ parents: 1, interior: 'here' }, acc],
				{ balance: balance, status: 'Liquid', reason: 'Consumer', extra: null },
			]),
		},
	}
}

export function setSwapPair() {
	return {
		assetSwap: {
			swapPair: {
				// Todo: replace with the actual address
				poolAccountId: '5DPiZzQQdoJJucxGMCgrJEdeUkLfPs6fndeCMA1E4ZgAkWyh',
				remoteAssetBalance: '100000000000000000000',
				remoteAssetId: {
					V3: {
						Concrete: {
							parents: 2,
							interior: {
								X2: [
									{ GlobalConsensus: { Ethereum: { chainId: 11155111 } } },
									// Todo: replace with the actual address
									{
										AccountKey20: {
											network: null,
											key: '0x06012c8cf97bead5deae237070f9587f8e7a266d',
										},
									},
								],
							},
						},
					},
				},

				remoteFee: {
					V3: {
						id: {
							concrete: {
								parents: 1,
								interior: 'Here',
							},
						},
						fun: { Fungible: 1 },
					},
				},
				remoteReserveLocation: {
					V3: {
						parents: 1,
						interior: { X1: { Parachain: { id: 1000 } } },
					},
				},
				status: 'Running',
			},
		},
	}
}

/// Spiritnet ParaId
export const paraId = 2086
export const KILT = { Concrete: { parents: 0, interior: 'Here' } }

/// Sibling sovereign account
export const siblingSovereignAccount = '5Eg2fnshxV9kofpcNEFE7azHLAjcCtpNkbsH3kkWZasYUVKs'
