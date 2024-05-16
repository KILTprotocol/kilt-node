import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as SpiritnetConfig from '../../network/spiritnet.js'
import * as HydraDxConfig from '../../network/hydraDx.js'
import { KILT, initialBalanceKILT, keysAlice } from '../../utils.js'
import { spiritnetContext, hydradxContext, getFreeBalanceSpiritnet, getFreeBalanceHydraDxKilt } from '../index.js'
import { getAccountLocationV3, getNativeAssetIdLocation, getSiblingLocation } from '../../network/utils.js'
import { checkBalance, checkBalanceInRange, createBlock, hexAddress, setStorage } from '../utils.js'

const KILT_ASSET_V3 = { V3: [getNativeAssetIdLocation(KILT)] }

test('Limited Reserve V3 Transfers from Spiritnet Account Alice -> HydraDx Account Alice', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILT tokens
	await setStorage(
		spiritnetContext,
		SpiritnetConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT)
	)

	// Balance of the hydraDx sovereign account before the transfer
	const hydraDxSovereignAccountBalanceBeforeTransfer = await getFreeBalanceSpiritnet(
		SpiritnetConfig.hydraDxSovereignAccount
	)

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalanceSpiritnet, keysAlice.address, expect, initialBalanceKILT)
	// Alice should have NO KILT on HydraDx
	await checkBalance(getFreeBalanceHydraDxKilt, keysAlice.address, expect, BigInt(0))

	const aliceAddress = hexAddress(keysAlice.address)
	const hydraDxDestination = { V3: getSiblingLocation(HydraDxConfig.paraId) }
	const beneficiary = getAccountLocationV3(aliceAddress)

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, KILT_ASSET_V3, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Check sender state
	await createBlock(spiritnetContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	// check balance. The sovereign account should hold one additional KILT.
	await checkBalance(
		getFreeBalanceSpiritnet,
		SpiritnetConfig.hydraDxSovereignAccount,
		expect,
		hydraDxSovereignAccountBalanceBeforeTransfer + KILT
	)

	// check balance sender
	// Equal to `initialBalanceKILT - KILT` - tx fees
	await checkBalanceInRange(getFreeBalanceSpiritnet, keysAlice.address, expect, [
		BigInt('98999830999996320'),
		BigInt('98999830999996321'),
	])

	// Check receiver state
	await createBlock(hydradxContext)

	// Check events receiver
	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// check balance receiver
	// check balance. Equal to `KILT` - tx fees
	await checkBalanceInRange(getFreeBalanceHydraDxKilt, aliceAddress, expect, [
		BigInt(996349465529793),
		BigInt(996349465529796),
	])
}, 20_000)
