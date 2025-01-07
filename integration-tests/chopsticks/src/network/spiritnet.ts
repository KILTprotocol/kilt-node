import { SetupOption } from '@acala-network/chopsticks-testing'

import { initialBalanceKILT, toNumber } from '../helper/utils.js'

/// Options used to create the Spiritnet context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) =>
	({
		endpoint: process.env.SPIRITNET_WS || 'wss://kilt.ibp.network',
		db: './db/spiritnet.db.sqlite',
		port: toNumber(process.env.SPIRITNET_PORT),
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

/// Spiritnet ParaId
export const paraId = 2086
export const KILT = { Concrete: { parents: 0, interior: 'Here' } }

/// Sibling sovereign account
export const sovereignAccountOnSiblingChains = '5Eg2fnshxV9kofpcNEFE7azHLAjcCtpNkbsH3kkWZasYUVKs'
