import { describe, beforeEach, test, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../../network/utils.js'
import { calculateTxFees, getPaidXcmFees, hexAddress } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../../network/types.js'
import { isSwitchPaused } from '../../index.js'
import { spinUpNetwork, tearDownNetwork } from '../../../utils.js'

describe.each(testCases)(
	'Send Relay token while switch paused',

	({ account, query, txContext, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc } = config

		// Create the network context
		beforeEach(async () => {
			const { parachainContexts, relayChainContext } = await spinUpNetwork(config)
			senderContext = parachainContexts[0]
			receiverContext = parachainContexts[1]
			relayContext = relayChainContext
			senderAccount = account
		})

		// Shut down the network
		afterEach(async () => await tearDownNetwork([receiverContext, senderContext, relayContext]))

		test(desc, async ({ expect }) => {
			const { checkSystemEvents, checkEvents } = withExpect(expect)
			const { tx, balanceToTransfer, events } = txContext

			// pre condition checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const balanceBeforeTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(0n)
			expect(balanceBeforeTxSender).toBeGreaterThan(0n)
			expect(await isSwitchPaused(receiverContext)).toBe(true)

			// action
			const rawTx = tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString())
			const events1 = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process msg
			await createBlock(receiverContext)

			// post condition checks
			// check balance movement on sender chain.
			const txFees = await calculateTxFees(rawTx, senderAccount)
			const xcmFees = await getPaidXcmFees(await events1.events)
			const balanceAfterTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))

			expect(balanceAfterTxSender).toBe(balanceBeforeTxSender - balanceToTransfer - txFees - xcmFees)

			// Tx should fail on receiver
			const balanceAfterTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceAfterTx).toBe(0n)

			// check events
			await Promise.all(
				events.sender.map((pallet) =>
					checkEvents(events1, pallet).toMatchSnapshot(
						`send funds from relay chain ${JSON.stringify(pallet)}`
					)
				)
			)

			await Promise.all(
				events.receiver.map((pallet) =>
					checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`receive relay chain funds on receiver chain ${JSON.stringify(pallet)}`
					)
				)
			)
		})
	}
)
