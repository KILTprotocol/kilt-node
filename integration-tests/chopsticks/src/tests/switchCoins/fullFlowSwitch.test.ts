import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assethub.js'
import { KILT, ROC, initialBalanceKILT, initialBalanceROC, keysAlice, keysCharlie } from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress, checkBalanceInRange } from '../utils.js'
import { getAccountLocationV3, getRelayNativeAssetIdLocation, getSiblingLocation } from '../../network/utils.js'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

/**
 * Full e2e flow between Peregrine and AssetHub
 *
 * 1. Send ROCs from AssetHub to Peregrine
 * 2. Switch KILTs on Peregrine
 * 3. Send eKILTs back to AssetHub
 * 4. Send ROCs back to AssetHub (Currently not implemented)
 */
test('Full e2e tests', async ({ expect }) => {
	const { checkEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	// 1. send ROCs 2 Peregrine

	const peregrineDestination = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const beneficiary1 = getAccountLocationV3(hexAddress(keysAlice.address))
	const rocAsset = { V3: [getRelayNativeAssetIdLocation(ROC)] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary1, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events1 = await sendTransaction(signedTx1)

	await createBlock(assethubContext)

	await checkEvents(events1, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	await checkEvents(events1, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	await checkEvents(events1, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	await createBlock(peregrineContext)

	// Alice should have some Rocs on Peregrine

	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))

	// 2. switch KILTs

	const balanceToTransfer = initialBalanceKILT / BigInt(2)
	const beneficiary = getAccountLocationV3(hexAddress(keysAlice.address))

	const signedTx2 = peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAsync(keysAlice)

	const events2 = await sendTransaction(signedTx2)

	await createBlock(peregrineContext)
	await checkEvents(events2, 'assetSwitchPool1').toMatchSnapshot('Switch events assetSwitchPool pallet')

	await createBlock(assethubContext)
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, balanceToTransfer)

	// 3. send eKILTs back
	const dest = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const remoteFeeId = { V3: { Concrete: AssetHubConfig.eKiltLocation } }
	const funds = {
		V3: [
			{
				id: { Concrete: AssetHubConfig.eKiltLocation },
				fun: { Fungible: balanceToTransfer / BigInt(2) },
			},
		],
	}

	const xcmMessage = {
		V3: [
			{
				DepositAsset: {
					assets: { Wild: 'All' },
					beneficiary: {
						parents: 0,
						interior: {
							X1: {
								AccountId32: {
									id: hexAddress(keysAlice.address),
								},
							},
						},
					},
				},
			},
		],
	}

	const signedTx3 = assethubContext.api.tx.polkadotXcm
		.transferAssetsUsingTypeAndThen(
			dest,
			funds,
			'LocalReserve',
			remoteFeeId,
			'LocalReserve',
			xcmMessage,
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events3 = await sendTransaction(signedTx3)

	await createBlock(assethubContext)

	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, KILT * BigInt(25))

	await checkEvents(events3, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'Sending eKILTs back'
	)

	await createBlock(peregrineContext)

	await checkBalanceInRange(getFreeBalancePeregrine, keysAlice.address, expect, [
		BigInt(74) * KILT,
		BigInt(75) * KILT,
	])

	// 4. send ROCs back TODO: implement
}, 20_000)
