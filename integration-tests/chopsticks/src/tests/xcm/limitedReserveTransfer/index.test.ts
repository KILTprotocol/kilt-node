/* eslint-disable @typescript-eslint/no-explicit-any */
import { describe, beforeEach, it, afterEach, ExpectStatic } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import { createBlock, hexAddress, setStorage } from '../../utils.js'

import { testPairsLimitedReserveTransfers } from './config.js'
import { Config } from '../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../index.js'
import type { KeyringPair } from '@polkadot/keyring/types'

describe.each(testPairsLimitedReserveTransfers)(
	'Limited Reserve Transfers',
	{ sequential: true, timeout: 30_000 },
	async ({ blockchain, storage, accounts, query, sovereignAccount, test, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		let receiverAccount: KeyringPair
		const { desc, precision } = config

		beforeEach(async () => {
			const { receiver, sender, relay } = blockchain

			const { receiverChainContext, senderChainContext, relayChainContext } = await setupNetwork(
				relay,
				sender,
				receiver
			)

			relayContext = relayChainContext
			senderContext = senderChainContext
			receiverContext = receiverChainContext

			const { receiverStorage, senderStorage, relayStorage } = storage
			await setStorage(senderContext, senderStorage)
			await setStorage(receiverContext, receiverStorage)
			await setStorage(relayContext, relayStorage)

			const { senderAccount: a, receiverAccount: b } = accounts
			senderAccount = a
			receiverAccount = b
		}, 20_000)

		afterEach(async () => {
			try {
				await shutDownNetwork([senderContext, receiverContext, relayContext])
			} catch (error) {
				console.error(error)
			}
		})

		it(desc, { timeout: 10_000, retry: 3 }, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)

			// test parameters
			const { pallets, tx, balanceToTransfer } = test

			// Balance of the receiver sovereign account before the transfer
			const receiverSovereignAccountBalanceBeforeTransfer = await query.sender(
				senderContext,
				sovereignAccount.sender
			)

			const initialBalanceReceiver = await query.receiver(receiverContext, receiverAccount.address)

			// Check initial balance receiver should be zero
			expect(initialBalanceReceiver).toBe(BigInt(0))

			const signedTx = tx(
				senderContext,
				hexAddress(receiverAccount.address),
				balanceToTransfer.toString()
			).signAsync(senderAccount)

			const events = await sendTransaction(signedTx)

			// check sender state
			await createBlock(senderContext)

			pallets.sender.map((pallet) =>
				checkEvents(events, pallet).toMatchSnapshot(`sender events ${JSON.stringify(pallet)}`)
			)

			const balanceSenderAfterTransfer = await query.sender(senderContext, senderAccount.address)
			const receiverSovereignAccountBalanceAfterTransfer = await query.sender(
				senderContext,
				sovereignAccount.sender
			)
			expect(receiverSovereignAccountBalanceAfterTransfer).toBe(
				receiverSovereignAccountBalanceBeforeTransfer + BigInt(balanceToTransfer)
			)

			checkBalanceInRange(balanceSenderAfterTransfer, expect, precision)

			// check receiver state
			await createBlock(receiverContext)

			pallets.receiver.map((pallet) =>
				checkSystemEvents(receiverContext, pallet).toMatchSnapshot(`receiver events ${JSON.stringify(pallet)}`)
			)

			const balanceReceiverAfterTransfer = await query.receiver(receiverContext, receiverAccount.address)

			checkBalanceInRange(balanceReceiverAfterTransfer, expect, precision)
		})
	}
)

// Check Balance is in range
function checkBalanceInRange(receivedBalance: bigint, expect: ExpectStatic, precision: bigint) {
	if (precision < BigInt(0) || precision > BigInt(100)) {
		throw new Error('Precision must be between 0 and 100')
	}

	const lowerBound = (receivedBalance * precision) / BigInt(100)
	expect(receivedBalance).toBeLessThanOrEqual(receivedBalance)
	expect(receivedBalance).toBeGreaterThanOrEqual(lowerBound)
}
