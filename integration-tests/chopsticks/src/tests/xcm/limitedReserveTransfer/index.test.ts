import { describe, beforeEach, it, afterEach } from 'vitest'
import type { KeyringPair } from '@polkadot/keyring/types'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import { createBlock, setStorage } from '../../../network/utils.js'
import { validateBalanceWithPrecision, hexAddress } from '../../../helper/utils.js'
import { testPairsLimitedReserveTransfers } from './config.js'
import { Config } from '../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../network/utils.js'

describe.each(testPairsLimitedReserveTransfers)(
	'Limited Reserve Transfers',
	{ timeout: 30_000, skip: true },
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
				if (!(error instanceof TypeError)) {
					console.error(error)
				}
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

			const initialBalanceSender = await query.sender(senderContext, senderAccount.address)

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

			validateBalanceWithPrecision(
				initialBalanceSender,
				balanceSenderAfterTransfer,
				balanceToTransfer,
				expect,
				precision
			)

			// check receiver state
			await createBlock(receiverContext)

			pallets.receiver.map((pallet) =>
				checkSystemEvents(receiverContext, pallet).toMatchSnapshot(`receiver events ${JSON.stringify(pallet)}`)
			)

			const balanceReceiverAfterTransfer = await query.receiver(receiverContext, receiverAccount.address)

			validateBalanceWithPrecision(
				initialBalanceReceiver,
				balanceReceiverAfterTransfer,
				balanceToTransfer,
				expect,
				precision
			)
		})
	}
)
