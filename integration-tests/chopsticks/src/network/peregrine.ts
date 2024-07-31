import { SetupOption, setupContext } from '@acala-network/chopsticks-testing'

import type { Config } from './types.js'
import { ROC, initialBalanceKILT, initialBalanceROC, toNumber } from '../utils.js'

/// Options used to create the Peregrine context
const options = {
	endpoint: process.env.PEREGRINE_WS || 'wss://peregrine.kilt.io',
	db: './db/peregrine.db.sqlite',
	port: toNumber(process.env.PEREGRINE_PORT) || 9004,
	wasmOverride: process.env.PEREGRINE_WASM_OVERRIDE || undefined,
	runtimeLogLevel: 5,
} as SetupOption

/// Assigns the native tokens to an accounts
export function assignNativeTokensToAccounts(addr: string[], balance: bigint = initialBalanceKILT) {
	return {
		System: {
			Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
		},
	}
}

export function setSafeXcmVersion3() {
	return {
		polkadotXcm: {
			safeXcmVersion: 3,
		},
	}
}

export function createAndAssignRocs(manager: string, addr: string[], balance: bigint = initialBalanceROC) {
	return {
		fungibles: {
			asset: [
				[
					[ROC_LOCATION],
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

export function setSudoKey(sudoKey: string) {
	return {
		Sudo: {
			key: sudoKey,
		},
	}
}

export function setSwitchPair(
	remoteAssetBalance: bigint = initialRemoteAssetBalance,
	poolAccountId: string = initialPoolAccountId,
	status: 'Running' | 'Paused' = 'Running'
) {
	return {
		assetSwitchPool1: {
			SwitchPair: {
				poolAccountId,
				remoteAssetBalance,
				remoteAssetId: {
					V3: {
						Concrete: {
							parents: 2,
							interior: {
								X2: [
									{ GlobalConsensus: { Ethereum: { chainId: 11155111 } } },
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
						fun: { Fungible: remoteFee },
					},
				},
				remoteReserveLocation: {
					V3: {
						parents: 1,
						interior: { X1: { Parachain: { id: 1000 } } },
					},
				},
				status,
			},
		},
		System: {
			Account: [[[poolAccountId], { providers: 1, data: { free: remoteAssetBalance } }]],
		},
	}
}

/// Peregrine ParaId
export const paraId = 2086
export const PILT = { Concrete: { parents: 0, interior: 'Here' } }
export const ROC_LOCATION = {
	parents: 1,
	interior: 'Here',
}
// 0.1 ROC as remote fee
export const remoteFee = ROC / BigInt(10)

/// Sibling sovereign account for other chains
export const siblingSovereignAccount = '5Eg2fnshxV9kofpcNEFE7azHLAjcCtpNkbsH3kkWZasYUVKs'

export const initialPoolAccountId = '4nv4phaKc4EcwENdRERuMF79ZSSB5xvnAk3zNySSbVbXhSwS'
export const initialRemoteAssetBalance = BigInt('100000000000000000000')

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
