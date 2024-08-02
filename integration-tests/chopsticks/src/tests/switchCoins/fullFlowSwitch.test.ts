import { test } from 'vitest'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import {
	KILT,
	ROC,
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysCharlie,
} from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
	checkSwitchPalletInvariant,
} from '../index.js'
import {
	checkBalance,
	createBlock,
	setStorage,
	hexAddress,
	checkBalanceInRange,
	getXcmMessageV4ToSendEkilt,
} from '../utils.js'
import { getAccountLocationV4, getRelayNativeAssetIdLocationV4, getSiblingLocationV4 } from '../../network/utils.js'
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
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	const switchParameters = getAssetSwitchParameters()

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(switchParameters))

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [
			[PeregrineConfig.siblingSovereignAccount, switchParameters.sovereignSupply],
		]),
	})

	// 1. send ROCs 2 Peregrine

	const peregrineDestination = { V4: getSiblingLocationV4(PeregrineConfig.paraId) }
	const beneficiary = getAccountLocationV4(hexAddress(keysAlice.address))
	const rocAsset = { V4: [getRelayNativeAssetIdLocationV4((ROC * BigInt(2)).toString())] }

	const signedTx1 = assethubContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(peregrineDestination, beneficiary, rocAsset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events1 = await sendTransaction(signedTx1)
	await createBlock(assethubContext)

	await checkEvents(events1, 'xcmpQueue').toMatchSnapshot(
		`sender AssetHub::xcmpQueue::[XcmpMessageSent]'  ${JSON.stringify(rocAsset)}`
	)
	await checkEvents(events1, 'polkadotXcm').toMatchSnapshot('sender AssetHub::polkadotXcm::[FeesPaid,Attempted,Sent]')
	await checkEvents(events1, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot(
		'sender Assethub::balances::[Withdraw]'
	)

	await createBlock(peregrineContext)

	// Alice should have some Rocs on Peregrine

	const aliceRocBalance = await getFreeRocPeregrine(keysAlice.address)
	expect(aliceRocBalance).toBeGreaterThan(BigInt(0))

	// 2. switch KILTs

	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	const signedTx2 = peregrineContext.api.tx.assetSwitchPool1
		.switch(balanceToTransfer.toString(), beneficiary)
		.signAsync(keysAlice)

	const events2 = await sendTransaction(signedTx2)

	await createBlock(peregrineContext)
	await checkEvents(events2, 'assetSwitchPool1').toMatchSnapshot(
		'receiver Peregrine::assetSwitchPool1::[LocalToRemoteSwitchExecuted]'
	)

	await createBlock(assethubContext)
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, balanceToTransfer)

	// 3. send eKILTs back
	const dest = { V4: getSiblingLocationV4(PeregrineConfig.paraId) }
	const remoteFeeId = { V4: AssetHubConfig.eKiltLocation }
	const funds = {
		V4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: balanceToTransfer / BigInt(2) },
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
			getXcmMessageV4ToSendEkilt(keysAlice.address),
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events3 = await sendTransaction(signedTx3)

	await createBlock(assethubContext)

	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, KILT * BigInt(25))

	await checkEvents(events3, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender AssetHub::foreignAssets::[Transferred]'
	)

	await createBlock(peregrineContext)

	await checkBalanceInRange(getFreeBalancePeregrine, keysAlice.address, expect, [
		BigInt(74) * KILT,
		BigInt(75) * KILT,
	])

	// 4. send ROCs back TODO: implement

	// const assetHubDestination = { V4: getSiblingLocation(AssetHubConfig.paraId) }

	// const assets = { V4: [getRelayNativeAssetIdLocation(ROC.toString())] }

	// const signedTx4 = peregrineContext.api.tx.polkadotXcm
	// 	.transferAssets(assetHubDestination, beneficiary, assets, 0, 'Unlimited')
	// 	.signAsync(keysAlice)

	// const events4 = await sendTransaction(signedTx4)
	// console.log(events4)

	await checkSwitchPalletInvariant(expect)
}, 20_000)
