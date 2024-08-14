import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../../../network/peregrine.js'
import * as RococoConfig from '../../../../network/rococo.js'
import * as AssetHubConfig from '../../../../network/assetHub.js'
import {
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysCharlie,
	ROC,
} from '../../../../utils.js'
import { peregrineContext, getFreeBalancePeregrine, getFreeRocPeregrine, rococoContext } from '../../../index.js'
import { checkBalance, createBlock, setStorage, hexAddress } from '../../../utils.js'
import {
	getAccountLocationV3,
	getChildLocation,
	getNativeAssetIdLocationV3,
	getSiblingLocationV4,
} from '../../../../network/utils.js'

test('Send DOTs from Relay 2 Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	const feeAmount = (ROC * BigInt(10)) / BigInt(100)
	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion4(),
	})

	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(getAssetSwitchParameters(), remoteAssetId, remoteXcmFeeId, remoteReserveLocation)
	)

	await setStorage(rococoContext, RococoConfig.assignNativeTokensToAccounts([keysAlice.address]))

	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))

	const balanceToTransfer = initialBalanceROC / BigInt(2)

	const aliceAddress = hexAddress(keysAlice.address)
	const hydraDxDestination = { V3: getChildLocation(PeregrineConfig.paraId) }
	const beneficiary = getAccountLocationV3(aliceAddress)
	const assetToTransfer = { V3: [getNativeAssetIdLocationV3(balanceToTransfer)] }

	const signedTx = rococoContext.api.tx.xcmPallet
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, assetToTransfer, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(rococoContext)

	checkEvents(events, 'xcmPallet').toMatchSnapshot('sender Rococo::xcmPallet::[XcmMessageSent]')

	await createBlock(peregrineContext)

	// Barrier will block execution. No event will be emitted.
	await checkSystemEvents(peregrineContext, {
		section: 'messageQueue',
		method: 'ProcessingFailed',
	}).toMatchSnapshot('receiver Peregrine::messageQueue::[ProcessingFailed]')

	// Alice should still have no balance
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))
}, 20_00000)
