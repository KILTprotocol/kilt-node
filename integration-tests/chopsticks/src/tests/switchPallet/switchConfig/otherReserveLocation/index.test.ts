import { describe, beforeEach, it, afterEach } from 'vitest'
import { withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, scheduleTx } from '../../../../network/utils.js'
import { hexAddress, keysAlice } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../../../network/types.js'
import { spinUpNetwork, tearDownNetwork } from '../../../utils.js'

describe.each(testCases)(
	'Switch other reserve location',

	({ account, query, txContext, config }) => {
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc } = config

		// Create the network context
		beforeEach(async () => {
			const { parachainContexts, relayChainContext } = await spinUpNetwork(config)

			relayContext = relayChainContext
			receiverContext = parachainContexts[0]
			senderAccount = account
		})

		// Shut down the network
		afterEach(async () => {
			await tearDownNetwork([receiverContext, relayContext])
		})

		it(desc, async ({ expect }) => {
			const { checkSystemEvents } = withExpect(expect)
			const { tx, balanceToTransfer, events, message } = txContext

			// precondition checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(0n)

			// action
			// schedule tx
			const rawTx = tx(relayContext, message(balanceToTransfer.toString(), keysAlice.address))
			await scheduleTx(relayContext, rawTx.method.toHex())
			// process tx
			await createBlock(relayContext)
			// process msg
			await createBlock(receiverContext)

			// post condition checks
			const balanceAfterTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceAfterTx).toBe(0n)

			// check events
			events.sender.map(
				async (pallet) =>
					await checkSystemEvents(relayContext, pallet).toMatchSnapshot(
						`Switch eKILTs from untrusted location sender: ${JSON.stringify(pallet)}`
					)
			)

			events.receiver.map(
				async (pallet) =>
					await checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`Switch eKILTs from untrusted location receiver: ${JSON.stringify(pallet)}`
					)
			)
		})
	}
)
