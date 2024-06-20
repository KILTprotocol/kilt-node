import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../network/utils.js'
import { hexAddress, validateBalanceWithPrecision } from '../../../helper/utils.js'
import { testPairsWithdrawAssets } from './config.js'
import { Config } from '../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../network/utils.js'

describe.each(testPairsWithdrawAssets)(
	'Withdraw Asset',
	{ timeout: 30_000 },
	async ({ network, storage, accounts, query, sovereignAccount, txContext, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		let receiverAccount: KeyringPair
		const { desc, precision } = config

		beforeEach(async () => {
			const { receiver, sender, relay } = network

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

			const { pallets, tx, balanceToTransfer } = txContext

			// Balance of the receiver sovereign account before the transfer
			const receiverSovereignAccountBalanceBeforeTransfer = await query.receiver(
				receiverContext,
				sovereignAccount.receiver
			)

			const balanceSenderBeforeTransfer = await query.sender(senderContext, senderAccount.address)

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

			const removedBalance = balanceToTransfer * BigInt(-1)

			validateBalanceWithPrecision(
				balanceSenderBeforeTransfer,
				balanceSenderAfterTransfer,
				removedBalance,
				expect,
				precision
			)

			// check receiver state
			await createBlock(receiverContext)

			const receiverSovereignAccountBalanceAfterTransfer = await query.receiver(
				receiverContext,
				sovereignAccount.receiver
			)
			expect(receiverSovereignAccountBalanceAfterTransfer).toBe(
				receiverSovereignAccountBalanceBeforeTransfer - BigInt(balanceToTransfer)
			)

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