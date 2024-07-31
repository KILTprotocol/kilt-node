import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../../../network/peregrine.js'
import * as RococoConfig from '../../../../network/rococo.js'
import { initialBalanceKILT, initialBalanceROC, keysAlice, keysCharlie } from '../../../../utils.js'
import { peregrineContext, getFreeBalancePeregrine, getFreeRocPeregrine, rococoContext } from '../../../index.js'
import { checkBalance, createBlock, setStorage, hexAddress } from '../../../utils.js'
import { getAccountLocationV3, getChildLocation, getNativeAssetIdLocation } from '../../../../network/utils.js'

// TODO: fix this test case. We only want to allow the transfer of DOTs from AH
test('Send DOTs from Relay 2 Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, []),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(rococoContext, RococoConfig.assignNativeTokensToAccounts([keysAlice.address]))

	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))

	const balanceToTransfer = initialBalanceROC / BigInt(2)

	const aliceAddress = hexAddress(keysAlice.address)
	const hydraDxDestination = { V3: getChildLocation(PeregrineConfig.paraId) }
	const beneficiary = getAccountLocationV3(aliceAddress)
	const assetToTransfer = { V3: [getNativeAssetIdLocation(balanceToTransfer)] }

	const signedTx = rococoContext.api.tx.xcmPallet
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, assetToTransfer, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(rococoContext)

	checkEvents(events, 'xcmPallet').toMatchSnapshot('sender events xcmPallet')

	await createBlock(peregrineContext)

	await checkSystemEvents(peregrineContext, {
		section: 'parachainSystem',
		method: 'DownwardMessagesReceived',
	}).toMatchSnapshot('receiver events parachainSystem pallet')

	await checkSystemEvents(peregrineContext, {
		section: 'dmpQueue',
		method: 'ExecutedDownward',
	}).toMatchSnapshot('receiver events dmpQueue pallet')

	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, BigInt(0))
}, 20_000)
