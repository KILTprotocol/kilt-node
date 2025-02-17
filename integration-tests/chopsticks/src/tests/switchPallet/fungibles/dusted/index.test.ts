import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../../network/utils.js'
import { calculateTxFees, hexAddress } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../../network/types.js'
import { spinUpNetwork, tearDownNetwork } from '../../../utils.js'

describe.each(testCases)('Dust account', async ({ account, query, txContext, config }) => {
	let context: Config
	let senderAccount: KeyringPair
	let receiverAccount: KeyringPair

	const { desc } = config

	// Create the network context
	beforeEach(async () => {
		const { parachainContexts } = await spinUpNetwork(config)
		context = parachainContexts[0]
		senderAccount = account.sender
		receiverAccount = account.receiver
	})

	// Shut down the network
	afterEach(async () => {
		await tearDownNetwork([context])
	})

	it(desc, async ({ expect }) => {
		const { checkSystemEvents } = withExpect(expect)
		const { balanceToTransfer, events, tx } = txContext

		// Pre condition checks
		const nativeBalance = await query.native(context, senderAccount.address)
		const foreignBalance = await query.foreign(context, senderAccount.address)
		const receiverNativeBalance = await query.native(context, receiverAccount.address)

		expect(nativeBalance).toBe(balanceToTransfer)
		expect(foreignBalance).toBeGreaterThan(0n)
		expect(receiverNativeBalance).toBe(0n)

		// action
		let rawTx = tx(context, hexAddress(receiverAccount.address), balanceToTransfer.toString())
		const txFees = await calculateTxFees(rawTx, senderAccount)
		rawTx = tx(context, hexAddress(receiverAccount.address), (balanceToTransfer - txFees).toString())

		await sendTransaction(rawTx.signAsync(senderAccount))

		// process tx
		await createBlock(context)

		// post condition checks

		// check balance movement
		const nativeBalanceAfter = await query.native(context, senderAccount.address)
		const foreignBalanceAfter = await query.foreign(context, senderAccount.address)
		const receiverNativeBalanceAfter = await query.native(context, receiverAccount.address)

		expect(nativeBalanceAfter).toBe(0n)
		// sender should keep the foreign balance
		expect(foreignBalanceAfter).toBeGreaterThan(0n)
		expect(receiverNativeBalanceAfter).toBe(balanceToTransfer - txFees)

		await Promise.all(
			events.map((pallet) =>
				checkSystemEvents(context, pallet).toMatchSnapshot(`Dusted accounts pallet: ${JSON.stringify(pallet)}`)
			)
		)
	})
})
