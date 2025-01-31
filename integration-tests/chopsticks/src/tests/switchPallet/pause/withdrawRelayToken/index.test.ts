import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../../network/utils.js'
import { hexAddress } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../../network/types.js'
import { isSwitchPaused } from '../../index.js'
import { spinUpNetwork, tearDownNetwork } from '../../../utils.js'

describe.each(testCases)(
	'Withdraw relay token while paused',

	async ({ account, query, txContext, config }) => {
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
			tearDownNetwork([receiverContext, senderContext, relayContext])
		})

		it(desc, async ({ expect }) => {
			const { checkSystemEvents, checkEvents } = withExpect(expect)

			const { tx, balanceToTransfer, events } = txContext

			// initial checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const balanceBeforeTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(BigInt(0))
			expect(balanceBeforeTxSender).toBeGreaterThan(BigInt(0))
			expect(await isSwitchPaused(senderContext)).toBe(true)

			const rawTx = tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString())
			const events1 = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process msg
			await createBlock(receiverContext)

			// check balance movement on sender chain.
			const balanceAfterTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceAfterTxSender).toBe(balanceBeforeTxSender - balanceToTransfer)

			const balanceAfterTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceAfterTx).toBeGreaterThan(BigInt(0))

			// check events
			events.sender.map(
				async (pallet) =>
					await checkEvents(events1, pallet).toMatchSnapshot(`Withdraw relay funds ${JSON.stringify(pallet)}`)
			)

			events.receiver.map(
				async (pallet) =>
					await checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`Receive relay funds ${JSON.stringify(pallet)}`
					)
			)
		})
	}
)
