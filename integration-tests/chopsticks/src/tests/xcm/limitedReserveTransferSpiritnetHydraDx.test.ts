import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

import * as SpiritnetConfig from '../../network/spiritnet.js'
import * as HydraDxConfig from '../../network/hydraDx.js'
import { keysAlice } from '../../helper.js'
import { spiritnetContext, hydradxContext, getFreeBalanceSpiritnet, getFreeBalanceHydraDxKilt } from '../index.js'

test.skip('Limited Reserve V3 Transfers from Spiritnet Account Bob -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// set storage
	await spiritnetContext.dev.setStorage(SpiritnetConfig.defaultStorage(keysAlice.address))
	await hydradxContext.dev.setStorage(HydraDxConfig.defaultStorage(keysAlice.address))

	// Create some new blocks to have consistent snapshots
	await new Promise((r) => setTimeout(r, 50))
	await spiritnetContext.dev.newBlock()
	await hydradxContext.dev.newBlock()

	// check initial balance
	const balanceSovereignAccountHydraDxBeforeTx = await getFreeBalanceSpiritnet(
		SpiritnetConfig.hydraDxSovereignAccount
	)
	expect(balanceSovereignAccountHydraDxBeforeTx).eq(0)

	const balanceToTransfer = 10e12
	const omniPoolAddress = u8aToHex(decodeAddress(HydraDxConfig.omnipoolAccount))

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(
			SpiritnetConfig.V3.hydraDxDestination,
			SpiritnetConfig.V3.hydraDxBeneficiary(omniPoolAddress),
			SpiritnetConfig.V3.nativeAssetIdLocation(balanceToTransfer),
			0,
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Produce new blocks
	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 50))
	await spiritnetContext.chain.newBlock()

	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 50))
	await hydradxContext.dev.newBlock()

	// Check events

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')

	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// check balance
	const balanceSovereignAccountHydraDxAfterTx = await getFreeBalanceSpiritnet(SpiritnetConfig.hydraDxSovereignAccount)
	expect(balanceSovereignAccountHydraDxAfterTx).eq(balanceToTransfer)

	let freeBalanceOmnipoolAccount = await getFreeBalanceHydraDxKilt(HydraDxConfig.omnipoolAccount)
	expect(freeBalanceOmnipoolAccount).eq(balanceToTransfer)
}, 20_000)

test.skip('Limited Reserve V2 Transfers from Spiritnet Account Bob -> HydraDx', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Set storage
	await spiritnetContext.dev.setStorage(SpiritnetConfig.defaultStorage(keysAlice.address))
	await hydradxContext.dev.setStorage(HydraDxConfig.defaultStorage(keysAlice.address))

	// Create some new blocks to have consistent snapshots
	await new Promise((r) => setTimeout(r, 50))
	await spiritnetContext.dev.newBlock()
	await hydradxContext.dev.newBlock()

	const balanceSovereignAccountHydraDxBeforeTx = await getFreeBalanceSpiritnet(
		SpiritnetConfig.hydraDxSovereignAccount
	)

	expect(balanceSovereignAccountHydraDxBeforeTx).eq(0)

	const balanceToTransfer = 10e12
	const omniPoolAddress = u8aToHex(decodeAddress(HydraDxConfig.omnipoolAccount))

	const signedTx = spiritnetContext.api.tx.polkadotXcm
		.limitedReserveTransferAssets(
			SpiritnetConfig.V2.hydraDxDestination,
			SpiritnetConfig.V2.hydraDxBeneficiary(omniPoolAddress),
			SpiritnetConfig.V2.nativeAssetIdLocation(balanceToTransfer),
			0,
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	// Produce new blocks
	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 50))
	await spiritnetContext.chain.newBlock()
	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 50))
	await hydradxContext.dev.newBlock()

	// Check events

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')

	checkSystemEvents(hydradxContext, { section: 'currencies', method: 'Deposited' }).toMatchSnapshot(
		'receiver events currencies'
	)
	checkSystemEvents(hydradxContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')

	// Check balance

	const balanceSovereignAccountHydraDxAfterTx = await getFreeBalanceSpiritnet(SpiritnetConfig.hydraDxSovereignAccount)
	expect(balanceSovereignAccountHydraDxAfterTx).eq(balanceToTransfer)

	let freeBalanceOmnipoolAccount = await getFreeBalanceHydraDxKilt(HydraDxConfig.omnipoolAccount)
	expect(freeBalanceOmnipoolAccount).eq(balanceToTransfer)
}, 20_000)
