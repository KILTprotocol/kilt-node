import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'
import { setTimeout } from 'timers/promises'

import * as SpiritnetConfig from '../../network/spiritnet.js'
import * as HydraDxConfig from '../../network/hydraDx.js'
import { KILT, keysAlice } from '../../utils.js'
import { spiritnetContext, hydradxContext, getFreeBalanceSpiritnet, getFreeBalanceHydraDxKilt } from '../index.js'
import { getAccountDestinationV3, getNativeAssetIdLocation, getSiblingDestination } from '../../network/utils.js'

test('Limited Reserve V3 Transfers from Spiritnet Account Alice -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// set storage
	await spiritnetContext.dev.setStorage(SpiritnetConfig.assignNativeTokensToAccount(keysAlice.address))
	await hydradxContext.dev.setStorage(HydraDxConfig.assignNativeTokensToAccount(keysAlice.address))

	// Create some new blocks to have consistent snapshots
	await setTimeout(50)
	await spiritnetContext.dev.newBlock()
	await hydradxContext.dev.newBlock()

	// check initial balance
	const balanceSovereignAccountHydraDxBeforeTx = await getFreeBalanceSpiritnet(
		SpiritnetConfig.hydraDxSovereignAccount
	)
	expect(balanceSovereignAccountHydraDxBeforeTx).eq(BigInt(0))
	const freeBalanceOmnipoolAccountBeforeTx = await getFreeBalanceHydraDxKilt(HydraDxConfig.omnipoolAccount)
	expect(freeBalanceOmnipoolAccountBeforeTx).eq(BigInt(0))

	const omniPoolAddress = u8aToHex(decodeAddress(HydraDxConfig.omnipoolAccount))
	const hydraDxDestination = { V3: getSiblingDestination(HydraDxConfig.paraId) }
	const beneficiary = getAccountDestinationV3(omniPoolAddress)

	const asset = { V3: getNativeAssetIdLocation(KILT) }

	console.log(asset)

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(hydraDxDestination, beneficiary, asset, 0, 'Unlimited')
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Produce new blocks
	// fixes api runtime disconnect warning
	await setTimeout(50)
	await spiritnetContext.chain.newBlock()
	await hydradxContext.dev.newBlock()

	// Check events

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// check balance
	const balanceSovereignAccountHydraDxAfterTx = await getFreeBalanceSpiritnet(SpiritnetConfig.hydraDxSovereignAccount)
	expect(balanceSovereignAccountHydraDxAfterTx).eq(BigInt(KILT))
	const freeBalanceOmnipoolAccountAfterTx = await getFreeBalanceHydraDxKilt(HydraDxConfig.omnipoolAccount)
	expect(freeBalanceOmnipoolAccountAfterTx).eq(BigInt(KILT))
}, 20_000)

test.skip('Limited Reserve V2 Transfers from Spiritnet Account Alice -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Set storage
	await spiritnetContext.dev.setStorage(SpiritnetConfig.defaultStorage(keysAlice.address))
	await hydradxContext.dev.setStorage(HydraDxConfig.defaultStorage(keysAlice.address))

	// Create some new blocks to have consistent snapshots
	await setTimeout(50)
	await spiritnetContext.dev.newBlock()
	await hydradxContext.dev.newBlock()

	// pre submit extrinsic checks
	const balanceSovereignAccountHydraDxBeforeTx = await getFreeBalanceSpiritnet(
		SpiritnetConfig.hydraDxSovereignAccount
	)
	expect(balanceSovereignAccountHydraDxBeforeTx).eq(BigInt(0))
	const freeBalanceOmnipoolAccountBeforeTx = await getFreeBalanceHydraDxKilt(HydraDxConfig.omnipoolAccount)
	expect(freeBalanceOmnipoolAccountBeforeTx).eq(BigInt(0))

	const omniPoolAddress = u8aToHex(decodeAddress(HydraDxConfig.omnipoolAccount))
	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(
			SpiritnetConfig.V2.hydraDxDestination,
			SpiritnetConfig.V2.hydraDxBeneficiary(omniPoolAddress),
			SpiritnetConfig.V2.nativeAssetIdLocation(KILT),
			0,
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Produce new blocks
	// fixes api runtime disconnect warning
	await setTimeout(50)
	await spiritnetContext.chain.newBlock()
	await hydradxContext.dev.newBlock()

	// Check events

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot('sender events Balances')

	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// Check balance

	const balanceSovereignAccountHydraDxAfterTx = await getFreeBalanceSpiritnet(SpiritnetConfig.hydraDxSovereignAccount)
	expect(balanceSovereignAccountHydraDxAfterTx).eq(KILT)
	const freeBalanceOmnipoolAccountAfterTx = await getFreeBalanceHydraDxKilt(HydraDxConfig.omnipoolAccount)
	expect(freeBalanceOmnipoolAccountAfterTx).eq(KILT)
}, 20_000)
