import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../../../network/peregrine.js'
import * as AssetHubConfig from '../../../../network/assetHub.js'
import { ROC, initialBalanceKILT, initialBalanceROC, keysAlice, keysBob, keysCharlie } from '../../../../utils.js'
import { peregrineContext, assethubContext, getFreeRocPeregrine, getFreeRocAssetHub } from '../../../index.js'
import { getSiblingLocation } from '../../../../network/utils.js'
import { checkBalance, createBlock, hexAddress, setStorage } from '../../../utils.js'

function getXcmMessage(amount: string | number, beneficiary: string) {
	return {
		V3: [
			{
				WithdrawAsset: [
					{
						id: { Concrete: { parents: 1, interior: 'Here' } },
						fun: { Fungible: amount },
					},
				],
			},
			{
				BuyExecution: {
					fees: {
						id: { Concrete: { parents: 1, interior: 'Here' } },
						fun: { Fungible: amount },
					},
					weightLimit: 'Unlimited',
				},
			},
			{
				InitiateReserveWithdraw: {
					assets: { Wild: 'All' },
					reserve: { parents: 0, interior: 'Here' },
					xcm: [
						{
							BuyExecution: {
								fees: {
									id: { Concrete: { parents: 1, interior: 'Here' } },
									fun: { Fungible: amount },
								},
								weightLimit: 'Unlimited',
							},
						},
						{
							DepositAsset: {
								assets: { Wild: 'All' },
								beneficiary: {
									parents: 0,
									interior: {
										X1: {
											AccountId32: {
												network: null,
												id: hexAddress(beneficiary),
											},
										},
									},
								},
							},
						},
					],
				},
			},
		],
	}
}

// TODO: Wait until: https://github.com/KILTprotocol/kilt-node/pull/655

test.skip('Initiate withdraw assets Peregrine Account Alice -> AH Account Bob', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	// Assign the sovereign account some ROCs
	await setStorage(
		assethubContext,
		AssetHubConfig.assignDotTokensToAccounts(
			[PeregrineConfig.siblingSovereignAccount, keysAlice.address],
			initialBalanceROC
		)
	)

	//const peregrineSovereignAccountBalanceBeforeTx = await getFreeRocAssetHub(PeregrineConfig.siblingSovereignAccount)

	// Alice should have some Rocs on Peregrine
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)

	// Bob should some ROCs on AH
	await checkBalance(getFreeRocAssetHub, keysBob.address, expect, BigInt(0))

	const assetHubDestination = { V3: getSiblingLocation(AssetHubConfig.paraId) }
	const xcmMessage = getXcmMessage(ROC.toString(), keysBob.address)

	const signedTx = peregrineContext.api.tx.polkadotXcm.send(assetHubDestination, xcmMessage).signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Check sender state
	await createBlock(peregrineContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
}, 20_000)
