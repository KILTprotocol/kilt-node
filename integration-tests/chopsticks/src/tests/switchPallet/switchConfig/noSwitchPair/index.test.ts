import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../../network/utils.js'
import { hexAddress } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../../../network/types.js'
import { spinUpNetwork, tearDownNetwork } from '../../../utils.js'

describe.each(testCases)(
	'Switch eKILTs while no pair set',

	({ account, query, txContext, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc } = config

		// Create the network context
		beforeEach(async () => {
			const { relayChainContext, parachainContexts } = await spinUpNetwork(config)
			relayContext = relayChainContext
			senderContext = parachainContexts[0]
			receiverContext = parachainContexts[1]
			senderAccount = account
		})

		// Shut down the network
		afterEach(async () => {
			await tearDownNetwork([receiverContext, senderContext, relayContext])
		})

		it(desc, async ({ expect }) => {
			const { checkSystemEvents, checkEvents } = withExpect(expect)
			const { tx, balanceToTransfer, events } = txContext

			// pre condition checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const balanceBeforeTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(0n)
			expect(balanceBeforeTxSender).toBeGreaterThan(0n)

			// action
			const rawTx = tx(senderContext, balanceToTransfer.toString())
			const events1 = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process msg
			await createBlock(receiverContext)

			// post condition checks
			// check balance movement on sender chain.
			const balanceAfterTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceAfterTxSender).toBe(balanceBeforeTxSender - balanceToTransfer)

			// Tx should fail on receiver
			const balanceAfterTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceAfterTx).toBe(0n)

			// check events
			events.sender.map(
				async (pallet) =>
					await checkEvents(events1, pallet).toMatchSnapshot(
						`Switch eKILTs on sender chain: ${JSON.stringify(pallet)}`
					)
			)

			events.receiver.map(
				async (pallet) =>
					await checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`Switch eKILTs on receiver chain: ${JSON.stringify(pallet)}`
					)
			)
		})
	}
)
