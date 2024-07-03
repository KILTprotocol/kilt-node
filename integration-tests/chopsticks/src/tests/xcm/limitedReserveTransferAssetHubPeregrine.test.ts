import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assethub.js'
import { ROC, initialBalanceKILT, initialBalanceROC, keysAlice, keysBob, keysCharlie } from '../../utils.js'
import { peregrineContext, assethubContext, getFreeRocPeregrine, getFreeRocAssetHub } from '../index.js'
import { getAccountLocationV3, getRelayNativeAssetIdLocation, getSiblingLocation } from '../../network/utils.js'
import { checkBalance, checkBalanceInRange, createBlock, hexAddress, setStorage } from '../utils.js'

const ROC_ASSET_V3 = { V3: [getRelayNativeAssetIdLocation(ROC)] }

test.skip('Limited Reserve V3 Transfers from AssetHub Account Alice -> Peregrine Account Bob', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILT tokens to create the account
	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.assignNativeTokensToAccounts(
			[keysBob.address, PeregrineConfig.poolAccountId],
			initialBalanceKILT
		),
		...PeregrineConfig.setSwapPair(),
	})

	// Give Alice some Rocs

	await setStorage(assethubContext, AssetHubConfig.assignDotTokensToAccounts([keysAlice.address], initialBalanceROC))

	const peregrineSovereignAccountBalanceBeforeTx = await getFreeRocAssetHub(PeregrineConfig.siblingSovereignAccount)

	// Alice should have no ROCs on Peregrine
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))

	// Alice should some ROCs on AH
	await checkBalance(getFreeRocAssetHub, keysAlice.address, expect, initialBalanceROC)

	const bobAddress = hexAddress(keysBob.address)
	const peregrineDestination = { V3: getSiblingLocation(PeregrineConfig.paraId) }
	const beneficiary = getAccountLocationV3(bobAddress)

	const signedTx = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary, ROC_ASSET_V3, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Check sender state
	await createBlock(assethubContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	// check balance. The sovereign account should hold one additional ROC.
	await checkBalance(
		getFreeRocAssetHub,
		PeregrineConfig.siblingSovereignAccount,
		expect,
		peregrineSovereignAccountBalanceBeforeTx + ROC
	)

	// check balance sender
	// Equal to `initialBalanceKILT - KILT` - tx fees
	await checkBalanceInRange(getFreeRocAssetHub, keysAlice.address, expect, [
		BigInt('98999830999996'),
		BigInt('98999830999996'),
	])

	// Check receiver state
	await createBlock(peregrineContext)

	// Check events receiver
	checkSystemEvents(peregrineContext, { section: 'fungibles', method: 'Issued' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(peregrineContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// check balance receiver
	// check balance. Equal to `KILT` - tx fees
	await checkBalanceInRange(getFreeRocPeregrine, bobAddress, expect, [BigInt(999999964195), BigInt(999999964296)])
}, 20_000)
