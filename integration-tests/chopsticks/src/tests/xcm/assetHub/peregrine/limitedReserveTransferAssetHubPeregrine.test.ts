import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../../../network/peregrine.js'
import * as AssetHubConfig from '../../../../network/assetHub.js'
import {
	ROC,
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysBob,
	keysCharlie,
} from '../../../../utils.js'
import { peregrineContext, assethubContext, getFreeRocPeregrine, getFreeRocAssetHub } from '../../../index.js'
import {
	getAccountLocationV4,
	getRelayNativeAssetIdLocationV4,
	getSiblingLocationV4,
} from '../../../../network/utils.js'
import { checkBalance, checkBalanceInRange, createBlock, hexAddress, setStorage } from '../../../utils.js'

const ROC_ASSET_V4 = { V4: [getRelayNativeAssetIdLocationV4(ROC)] }

test('Limited Reserve V4 Transfers from AssetHub Account Alice -> Peregrine Account Bob', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)
	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	// Assign alice some KILT tokens to create the account
	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.assignNativeTokensToAccounts([keysBob.address], initialBalanceKILT),
	})

	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(getAssetSwitchParameters(), remoteAssetId, remoteXcmFeeId, remoteReserveLocation)
	)

	// Give Alice some Rocs
	await setStorage(
		assethubContext,
		AssetHubConfig.assignDotTokensToAccountsAsStorage([keysAlice.address], initialBalanceROC)
	)

	const peregrineSovereignAccountBalanceBeforeTx = await getFreeRocAssetHub(PeregrineConfig.sovereignAccountAsSibling)

	// Bob should have no ROCs on Peregrine
	await checkBalance(getFreeRocPeregrine, keysBob.address, expect, BigInt(0))

	// Alice should some ROCs on AH
	await checkBalance(getFreeRocAssetHub, keysAlice.address, expect, initialBalanceROC)

	const bobAddress = hexAddress(keysBob.address)
	const peregrineDestination = getSiblingLocationV4(PeregrineConfig.paraId)
	const beneficiary = getAccountLocationV4(bobAddress)

	const signedTx = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary, ROC_ASSET_V4, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Check sender state
	await createBlock(assethubContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender AssetHub::xcmpQueue::[XcmMessageSent]')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender AssetHub::polkadotXcm::[FeesPaid,Attempted,Sent]')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot(
		'sender AssetHub::balances::[Withdraw]'
	)

	// check balance. The sovereign account should hold one additional ROC.
	await checkBalance(
		getFreeRocAssetHub,
		PeregrineConfig.sovereignAccountAsSibling,
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
		'receiver Peregrine::fungibles::[Issued]'
	)
	checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot('receiver Peregrine::messageQueue::[Processed]')

	// check balance receiver
	// check balance. Equal to `KILT` - tx fees
	await checkBalanceInRange(getFreeRocPeregrine, bobAddress, expect, [BigInt(999999964195), BigInt(999999964296)])
}, 20_000)
