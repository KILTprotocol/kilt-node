import { SetupOption, setupContext } from '@acala-network/chopsticks-testing'

import type { Config } from './types.js'
import { ROC, initialBalanceKILT, initialBalanceROC, toNumber } from '../utils.js'
import { AssetSwitchSupplyParameters } from '../types.js'

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

export function setSafeXcmVersion4() {
	return {
		polkadotXcm: {
			safeXcmVersion: 4,
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
						owner: manager,
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
	parameters: AssetSwitchSupplyParameters,
	remoteAssetId: Record<string, unknown>,
	remoteXcmFee: Record<string, unknown>,
	remoteReserveLocation: Record<string, unknown>,
	poolAccountId: string = initialPoolAccountId,
	status: 'Running' | 'Paused' = 'Running'
) {
	return {
		assetSwitchPool1: {
			SwitchPair: {
				poolAccountId,
				remoteAssetSovereignTotalBalance: parameters.sovereignSupply,
				remoteAssetCirculatingSupply: parameters.circulatingSupply,
				remoteAssetTotalSupply: parameters.totalSupply,
				remoteAssetId,
				remoteXcmFee,
				remoteReserveLocation,
				status,
			},
		},
		// the pool account needs at least as much fund to cover the circulating supply. Give him exactly that amount + ED.
		System: {
			Account: [[[poolAccountId], { providers: 1, data: { free: parameters.circulatingSupply } }]],
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
export const sovereignAccountAsSibling = '5Eg2fnshxV9kofpcNEFE7azHLAjcCtpNkbsH3kkWZasYUVKs'
// ED on Peregrine
export const existentialDeposit = BigInt('10000000000000')

export const initialPoolAccountId = '4nv4phaKc4EcwENdRERuMF79ZSSB5xvnAk3zNySSbVbXhSwS'

export async function getContext(): Promise<Config> {
	return setupContext(options)
}
