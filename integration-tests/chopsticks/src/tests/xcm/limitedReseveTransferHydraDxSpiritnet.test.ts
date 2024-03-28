import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as HydraDxConfig from '../../network/hydraDx.js'
import * as SpiritnetConfig from '../../network/spiritnet.js'
import { keysAlice, keysBob } from '../../helper.js'
import { getFreeBalanceHydraDxKilt, getFreeBalanceSpiritnet, hydradxContext, spiritnetContext } from '../index.js'
import { initBalance } from '../../network/utils.js'

test('Limited Reserve Transfers from HydraDx Account Bob -> Spiritnet', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Create some new blocks to have consistent snapshots
	await new Promise((r) => setTimeout(r, 50))
	await spiritnetContext.dev.newBlock()
	await hydradxContext.dev.newBlock()

	// Give the sovereign account of HydraDx some kilt coins.
	await spiritnetContext.dev.setStorage(SpiritnetConfig.defaultStorage(HydraDxConfig.sovereignAccount))
	await hydradxContext.dev.setStorage(HydraDxConfig.defaultStorage(keysBob.address))

	const balanceToTransfer = 10e9

	const aliceBalanceBeforeTx = await getFreeBalanceSpiritnet(keysAlice.address)

	expect(aliceBalanceBeforeTx).eq(0)

	const signedTx = hydradxContext.api.tx.xTokens
		.transfer(
			HydraDxConfig.kiltTokenId,
			balanceToTransfer,
			HydraDxConfig.spiritnetDestinationAccount(keysAlice.address),
			'Unlimited'
		)
		.signAsync(keysBob)

	const events = await sendTransaction(signedTx)

	// Produce a new Block
	// fixes api runtime disconnect warning
	await new Promise((r) => setTimeout(r, 50))
	await hydradxContext.chain.newBlock()
	await new Promise((r) => setTimeout(r, 50))
	await spiritnetContext.dev.newBlock()

	// Check Events
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'polkadotXcm').toMatchSnapshot('sender events xcm pallet')
	checkEvents(events, 'xTokens').toMatchSnapshot('sender events xcm pallet')

	checkSystemEvents(spiritnetContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')
	checkSystemEvents(spiritnetContext, 'polkadotXcm').toMatchSnapshot('receiver events polkadotXCM')
	checkSystemEvents(spiritnetContext, 'balances').toMatchSnapshot('receiver events balances')

	// Check Balance

	const balanceSovereignAccountHydraDxAfterTx = await getFreeBalanceSpiritnet(HydraDxConfig.sovereignAccount)
	expect(balanceSovereignAccountHydraDxAfterTx).eq(initBalance - balanceToTransfer)

	const balanceAliceSpiritnetAfterTx = await getFreeBalanceSpiritnet(keysAlice.address)
	expect(balanceAliceSpiritnetAfterTx).eq(balanceToTransfer)

	let balanceBobHydraDx = await getFreeBalanceHydraDxKilt(keysBob.address)
	expect(balanceBobHydraDx).eq(initBalance - balanceToTransfer)
}, 20_000)
