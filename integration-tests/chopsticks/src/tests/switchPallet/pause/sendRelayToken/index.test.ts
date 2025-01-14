import { describe, beforeEach, test, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../../network/utils.js'
import { calculateTxFees, getPaidXcmFees, hexAddress } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../../network/utils.js'

describe.skip.each(testCases)(
	'Send Relay token while switch paused',
	{ timeout: 30_000 },
	async ({ account, query, txContext, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc, network, storage } = config

		// Create the network context
		beforeEach(async () => {
			const { parachains, relay } = network

			const { parachainContexts, relayChainContext } = await setupNetwork(relay, parachains)
			const [senderChainContext, receiverChainContext] = parachainContexts

			relayContext = relayChainContext
			senderContext = senderChainContext
			receiverContext = receiverChainContext

			const { receiverStorage, senderStorage, relayStorage } = storage
			await setStorage(senderContext, senderStorage)
			await setStorage(receiverContext, receiverStorage)
			await setStorage(relayContext, relayStorage)

			senderAccount = account
		}, 20_000)

		// Shut down the network
		afterEach(async () => {
			try {
				await shutDownNetwork([receiverContext, senderContext, relayContext])
			} catch (error) {
				if (!(error instanceof TypeError)) {
					console.error(error)
				}
			}
		})

		test(desc, { timeout: 10_000, retry: 3 }, async ({ expect }) => {
			const { checkSystemEvents, checkEvents } = withExpect(expect)

			const { tx, balanceToTransfer, events } = txContext

			// inital checks
			const balanceBeforeTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const balanceBeforeTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))
			expect(balanceBeforeTx).toBe(BigInt(0))
			expect(balanceBeforeTxSender).toBeGreaterThan(BigInt(0))
			const rawTx = tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString())

			const events1 = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process msg
			await createBlock(receiverContext)

			// check balance movement on sender chain.
			const txFees = await calculateTxFees(rawTx, senderAccount)
			const xcmFees = await getPaidXcmFees(await events1.events)
			const balanceAfterTxSender = await query.sender(senderContext, hexAddress(senderAccount.address))

			expect(balanceAfterTxSender).toBe(balanceBeforeTxSender - balanceToTransfer - txFees - xcmFees)

			// Tx should fail on receiver
			const balanceAfterTx = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			expect(balanceAfterTx).toBe(BigInt(0))

			// check events
			events.sender.map(
				async (pallet) =>
					await checkEvents(events1, pallet).toMatchSnapshot(
						`${desc}: send funds from relay chain ${JSON.stringify(pallet)}`
					)
			)

			events.receiver.map(
				async (pallet) =>
					await checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`${desc}: receive relay chain funds on receiver chain ${JSON.stringify(pallet)}`
					)
			)
		})
	}
)
