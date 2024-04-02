import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'
import { setTimeout } from 'timers/promises'

import * as HydraDxConfig from '../../network/hydraDx.js'
import * as SpiritnetConfig from '../../network/spiritnet.js'
import { KILT, initialBalanceKILT, keysAlice, keysBob } from '../../utils.js'
import { getFreeBalanceHydraDxKilt, getFreeBalanceSpiritnet, hydradxContext, spiritnetContext } from '../index.js'

test('Limited Reserve Transfers from HydraDx Account Bob -> Spiritnet', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Create some new blocks to have consistent snapshots
	await setTimeout(50)
	await spiritnetContext.dev.newBlock()
	await hydradxContext.dev.newBlock()

	// Give the sovereign account of HydraDx some kilt coins.
	await spiritnetContext.dev.setStorage(
		SpiritnetConfig.defaultStorage(u8aToHex(decodeAddress(SpiritnetConfig.hydraDxSovereignAccount)))
	)
	await hydradxContext.dev.setStorage(HydraDxConfig.defaultStorage(keysBob.address))

	// check initial balance of alice
	const aliceBalanceBeforeTx = await getFreeBalanceSpiritnet(keysAlice.address)
	expect(aliceBalanceBeforeTx).eq(BigInt(0))

	const signedTx = hydradxContext.api.tx.xTokens
		.transfer(
			HydraDxConfig.kiltTokenId,
			KILT,
			HydraDxConfig.spiritnetDestinationAccount(keysAlice.address),
			'Unlimited'
		)
		.signAsync(keysBob)

	const events = await sendTransaction(signedTx)

	// Produce a new Block
	// fixes api runtime disconnect warning
	await setTimeout(50)
	await hydradxContext.chain.newBlock()
	await spiritnetContext.dev.newBlock()

	// Check Events HydraDx
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, { section: 'currencies', method: 'Withdrawn' }).toMatchSnapshot('sender events currencies')
	checkEvents(events, 'xTokens').toMatchSnapshot('sender events currencies')

	// check Events Spiritnet
	checkSystemEvents(spiritnetContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')
	checkSystemEvents(spiritnetContext, { section: 'balances', method: 'Withdraw' }).toMatchSnapshot(
		'receiver events Balances'
	)
	checkSystemEvents(spiritnetContext, { section: 'balances', method: 'Endowed' }).toMatchSnapshot(
		'receiver events Balances'
	)

	// Check Balance
	const balanceSovereignAccountHydraDxAfterTx = await getFreeBalanceSpiritnet(SpiritnetConfig.hydraDxSovereignAccount)
	expect(balanceSovereignAccountHydraDxAfterTx).eq(initialBalanceKILT - KILT)

	const balanceBobHydraDx = await getFreeBalanceHydraDxKilt(keysBob.address)
	expect(balanceBobHydraDx).eq(initialBalanceKILT - KILT)
}, 20_000)
