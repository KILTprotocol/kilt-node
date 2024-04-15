import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

import * as SpiritnetConfig from '../../network/spiritnet.js'
import * as HydraDxConfig from '../../network/hydraDx.js'
import { KILT, keysAlice } from '../../utils.js'
import { spiritnetContext, hydradxContext, getFreeBalanceSpiritnet, getFreeBalanceHydraDxKilt } from '../index.js'
import {
	getAccountDestinationV2,
	getAccountDestinationV3,
	getNativeAssetIdLocation,
	getSiblingDestination,
} from '../../network/utils.js'
import { checkBalanceAndExpectAmount, checkBalanceAndExpectZero, createBlock, setStorage } from '../utils.js'

const KILT_ASSET_V3 = { V3: [getNativeAssetIdLocation(KILT)] }
const KILT_ASSET_V2 = { V2: [getNativeAssetIdLocation(KILT)] }

test('Limited Reserve V3 Transfers from Spiritnet Account Alice -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// set storage
	await setStorage(spiritnetContext, SpiritnetConfig.assignNativeTokensToAccount(keysAlice.address))
	await setStorage(spiritnetContext, SpiritnetConfig.setSafeXcmVersion(3))
	await setStorage(hydradxContext, HydraDxConfig.assignNativeTokensToAccount(keysAlice.address))
	await setStorage(hydradxContext, HydraDxConfig.registerKilt())

	// check initial balance
	await checkBalanceAndExpectZero(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect)
	await checkBalanceAndExpectZero(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect)

	const omniPoolAddress = u8aToHex(decodeAddress(HydraDxConfig.omnipoolAccount))
	const hydraDxDestination = { V3: getSiblingDestination(HydraDxConfig.paraId) }
	const beneficiary = getAccountDestinationV3(omniPoolAddress)

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, KILT_ASSET_V3, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Produce new blocks
	await Promise.all([createBlock(spiritnetContext), createBlock(hydradxContext)])

	// Check events
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// check balance
	await checkBalanceAndExpectAmount(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect, KILT)
	await checkBalanceAndExpectAmount(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect, KILT)
}, 20_000)

test('Limited Reserve V2 Transfers from Spiritnet Account Alice -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Set storage
	await setStorage(spiritnetContext, SpiritnetConfig.assignNativeTokensToAccount(keysAlice.address))
	await setStorage(spiritnetContext, SpiritnetConfig.setSafeXcmVersion(3))
	await setStorage(hydradxContext, HydraDxConfig.assignNativeTokensToAccount(keysAlice.address))
	await setStorage(hydradxContext, HydraDxConfig.registerKilt())

	// check initial balance
	await checkBalanceAndExpectZero(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect)
	await checkBalanceAndExpectZero(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect)

	const omniPoolAddress = u8aToHex(decodeAddress(HydraDxConfig.omnipoolAccount))
	const hydraDxDestination = { V2: getSiblingDestination(HydraDxConfig.paraId) }
	const beneficiary = getAccountDestinationV2(omniPoolAddress)

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, KILT_ASSET_V2, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Produce new blocks
	await Promise.all([createBlock(spiritnetContext), createBlock(hydradxContext)])

	// Check events
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// check balance
	await checkBalanceAndExpectAmount(getFreeBalanceSpiritnet, SpiritnetConfig.hydraDxSovereignAccount, expect, KILT)
	await checkBalanceAndExpectAmount(getFreeBalanceHydraDxKilt, HydraDxConfig.omnipoolAccount, expect, KILT)
}, 20_000)
