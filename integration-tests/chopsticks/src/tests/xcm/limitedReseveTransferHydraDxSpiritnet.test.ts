import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

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
	await spiritnetContext.dev.setStorage(
		SpiritnetConfig.defaultStorage(u8aToHex(decodeAddress(SpiritnetConfig.hydraDxSovereignAccount)))
	)
	await hydradxContext.dev.setStorage(HydraDxConfig.defaultStorage(keysBob.address))

	// check initial balance of alice
	const aliceBalanceBeforeTx = await getFreeBalanceSpiritnet(keysAlice.address)
	expect(aliceBalanceBeforeTx).eq(0)

	const balanceToTransfer = 10e5

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
	checkEvents(events, { section: 'currencies', method: 'Withdrawn' }).toMatchSnapshot('sender events currencies')
	checkEvents(events, 'xTokens').toMatchSnapshot('sender events currencies')

	checkSystemEvents(spiritnetContext, 'xcmpQueue').toMatchSnapshot('receiver events xcmpQueue')
	checkSystemEvents(spiritnetContext, 'balances').toMatchSnapshot('receiver events polkadotXCM')

	// Check Balance
	const balanceSovereignAccountHydraDxAfterTx = await getFreeBalanceSpiritnet(SpiritnetConfig.hydraDxSovereignAccount)
	expect(balanceSovereignAccountHydraDxAfterTx).eq(initBalance - balanceToTransfer)

	console.log(keysAlice.address)

	const balanceBobHydraDx = await getFreeBalanceHydraDxKilt(keysBob.address)
	expect(balanceBobHydraDx).eq(initBalance - balanceToTransfer)

	await new Promise((r) => setTimeout(r, 50))
	await spiritnetContext.dev.newBlock()
	const balanceAliceSpiritnetAfterTx = await getFreeBalanceSpiritnet(
		'4qPZ8fv6BjGoGKzfx5LtBFnEUp2b5Q5C1ErrjBNGmoFTLNHG'
	)
	expect(balanceAliceSpiritnetAfterTx).eq(balanceToTransfer)
}, 20_000)
