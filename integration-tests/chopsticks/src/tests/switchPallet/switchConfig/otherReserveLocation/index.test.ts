import { describe, beforeEach, it, afterEach } from 'vitest'
import { withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, scheduleTx, setStorage } from '../../../../network/utils.js'
import { hexAddress, keysAlice } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../../network/utils.js'

describe.each(testCases)(
	'Switch other reserve location',
	{ timeout: 30_000 },
	async ({ account, query, txContext, config }) => {
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc, network, storage } = config

		// Create the network context
		beforeEach(async () => {
			const { parachains, relay } = network

			const { parachainContexts, relayChainContext } = await setupNetwork(relay, parachains)
			const [receiverChainContext] = parachainContexts

			relayContext = relayChainContext
			receiverContext = receiverChainContext

			const { receiverStorage, relayStorage } = storage
			await setStorage(receiverContext, receiverStorage)
			await setStorage(relayContext, relayStorage)

			senderAccount = account
		}, 20_000)

		// Shut down the network
		afterEach(async () => {
			try {
				await shutDownNetwork([receiverContext, relayContext])
			} catch (error) {
				if (!(error instanceof TypeError)) {
					console.error(error)
				}
			}
		})

		it(desc, { timeout: 10_000 }, async ({ expect }) => {
			const { checkSystemEvents } = withExpect(expect)
			const { tx, balanceToTransfer, events, message } = txContext

			// initial checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(BigInt(0))

			// schedule tx
			const rawTx = tx(relayContext, message(balanceToTransfer.toString(), keysAlice.address))
			await scheduleTx(relayContext, rawTx)
			// process tx
			await createBlock(relayContext)
			// process msg
			await createBlock(receiverContext)

			// Tx should fail on receiver. No balance movement.
			const balanceAfterTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceAfterTx).toBe(BigInt(0))

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
