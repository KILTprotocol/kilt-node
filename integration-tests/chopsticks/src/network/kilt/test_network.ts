import { SetupOption } from '@acala-network/chopsticks-testing'

import { initialBalanceKILT, toNumber } from '../../helper/utils.js'

/// Options used to create the Spiritnet context
export const getSetupOptions = ({
	blockNumber = undefined,
	wasmOverride = undefined,
}: {
	blockNumber?: number
	wasmOverride?: string
}) => {
	const random = (Math.random() + 1).toString(36).substring(7)
	return {
		endpoint: process.env.SPIRITNET_WS || 'wss://peregrine.kilt.io/parachain-public-ws/',
		db: `./db/spiritnet_main_${random}.db.sqlite`,
		port: toNumber(process.env.SPIRITNET_PORT),
		wasmOverride,
		blockNumber,
	} as SetupOption
}

export const storage = {
	/// Assigns the native tokens to an accounts
	assignNativeTokensToAccounts(addr: string[], balance: bigint = initialBalanceKILT) {
		return {
			System: {
				Account: addr.map((address) => [[address], { providers: 1, data: { free: balance } }]),
			},
		}
	},

	/// Sets the [technicalCommittee] and [council] governance to the given accounts
	setGovernance(addr: string[]) {
		return {
			technicalCommittee: { Members: addr },
			council: { Members: addr },
		}
	},
}

export const parachainInfo = {
	/// Spiritnet ParaId
	paraId: 2086,
	/// Sibling sovereign account
	sovereignAccountOnSiblingChains: '5Eg2fnshxV9kofpcNEFE7azHLAjcCtpNkbsH3kkWZasYUVKs',

	HERE: { Concrete: { parents: 0, interior: 'Here' } },
}
