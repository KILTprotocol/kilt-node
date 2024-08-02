/* eslint-disable @typescript-eslint/no-unused-vars */
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
	keysCharlie,
} from '../../../../utils.js'
import { peregrineContext, assethubContext, getFreeRocPeregrine, getFreeRocAssetHub } from '../../../index.js'
import {
	getAccountLocationV4,
	getRelayNativeAssetIdLocationV4,
	getSiblingLocationV4,
} from '../../../../network/utils.js'
import { checkBalanceInRange, createBlock, hexAddress, setStorage } from '../../../utils.js'

test('Initiate withdraw assets Peregrine Account Alice -> AH Account Alice', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address]),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	const switchParameters = getAssetSwitchParameters()

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(switchParameters))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts([PeregrineConfig.siblingSovereignAccount], initialBalanceROC),
	})

	// check initial state
	const balanceAliceRocPeregrineBeforeTx = await getFreeRocPeregrine(keysAlice.address)
	const balanceAliceRocAssetHubBeforeTx = await getFreeRocAssetHub(keysAlice.address)
	const balanceSovereignAccountAssetHubBeforeTx = await getFreeRocAssetHub(PeregrineConfig.siblingSovereignAccount)

	expect(balanceAliceRocPeregrineBeforeTx).toBe(initialBalanceROC)
	expect(balanceAliceRocAssetHubBeforeTx).toBe(BigInt(0))
	expect(balanceSovereignAccountAssetHubBeforeTx).toBe(initialBalanceROC)

	const assetHubDestination = { V4: getSiblingLocationV4(AssetHubConfig.paraId) }
	// We send 1 ROC
	const assets = { V4: [getRelayNativeAssetIdLocationV4(ROC.toString())] }
	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))

	const signedTx4 = peregrineContext.api.tx.polkadotXcm
		.transferAssets(assetHubDestination, beneficiary, assets, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events4 = await sendTransaction(signedTx4)
	await createBlock(peregrineContext)

	// The xcm message should be send to AH and the funds should be burned from user.
	await checkEvents(events4, 'fungibles').toMatchSnapshot('sender Peregrine::fungibles::[Burned]')
	await checkEvents(events4, 'xcmpQueue').toMatchSnapshot('sender Peregrine::xcmpQueue::[XcmMessageSent]')
	await checkEvents(events4, 'polkadotXcm').toMatchSnapshot(
		'sender Peregrine::polkadotXcm::[FeesPaid,Attempted,Sent]'
	)

	// Alice funds after the transaction
	const balanceAliceRocPeregrineAfterTx = await getFreeRocPeregrine(keysAlice.address)
	expect(balanceAliceRocPeregrineAfterTx).toBe(initialBalanceROC - ROC)

	// The funds should be burned from Sovereign account and minted to user.
	await createBlock(assethubContext)
	await checkSystemEvents(assethubContext, { section: 'balances', method: 'Burned' }).toMatchSnapshot(
		'receiver AssetHub::balances::Burned'
	)
	await checkSystemEvents(assethubContext, { section: 'balances', method: 'Minted' }).toMatchSnapshot(
		'receiver AssetHub::balances::Minted'
	)
	await checkSystemEvents(assethubContext, { section: 'balances', method: 'Endowed' }).toMatchSnapshot(
		'receiver AssetHub::balances::Endowed'
	)
	await checkSystemEvents(assethubContext, { section: 'messageQueue', method: 'Processed' }).toMatchSnapshot(
		'receiver AssetHub::messageQueue::Processed'
	)

	// state sovereign account
	const balanceSovereignAccountAssetHubAfterTx = await getFreeRocAssetHub(PeregrineConfig.siblingSovereignAccount)
	expect(balanceSovereignAccountAssetHubAfterTx).toBe(initialBalanceROC - ROC)

	// state alice on asset hub
	await checkBalanceInRange(getFreeRocPeregrine, keysAlice.address, expect, [BigInt(999999964195), ROC])
}, 20_000)
