import type { SetupOption } from '@acala-network/chopsticks-testing'

import { initialBalanceDOT, initialBalanceKILT, toNumber } from '../../helper/utils.js'

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
		endpoint: process.env.SPIRITNET_WS || 'wss://kilt.ibp.network',
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
	/// Assigns the relay token to the account.
	assignRelayTokensToAccounts(addr: string[], balance: bigint = initialBalanceDOT) {
		return {
			fungibles: {
				account: addr.map((acc) => [
					[{ parents: 1, interior: 'here' }, acc],
					{ balance: balance, status: 'Liquid', reason: 'Consumer', extra: null },
				]),
			},
		}
	},
	/// Pauses the switch pool. Default value are fetched from block 7,896,550 and will break the invariant check.
	pauseSwitch(
		remoteAssetCirculatingSupply = '1044350720000000000000',
		remoteAssetTotalSupply = '164000000000000000000000',
		remoteAssetSovereignTotalBalance = '162955649280000000000000'
	) {
		return {
			assetSwitchPool1: {
				switchPair: {
					status: 'Paused',
					poolAccount: '4pCvXiDJXzfms5G2Digp474mo3SJSsAWUBuAJpzcuZvvK8dt',
					remoteAssetCirculatingSupply,
					remoteAssetEd: 1,
					remoteAssetId: {
						V4: {
							parents: 2,
							interior: {
								X2: [
									{
										GlobalConsensus: {
											Ethereum: {
												chainId: 1,
											},
										},
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
					},
					remoteAssetTotalSupply,
					remoteReserveLocation: {
						V4: {
							parents: 1,
							interior: {
								X1: [
									{
										Parachain: 1000,
									},
								],
							},
						},
					},
					remoteXcmFee: {
						V4: {
							id: {
								parents: 1,
								interior: 'Here',
							},
							fun: {
								Fungible: 5000000000,
							},
						},
					},
					remoteAssetSovereignTotalBalance,
				},
			},
		}
	},
	/// Removes the switch pool
	removeSwitchPair() {
		return {
			assetSwitchPool1: {
				switchPair: {},
			},
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
