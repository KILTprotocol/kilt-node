/* eslint-disable @typescript-eslint/no-unused-vars */
import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../network/utils.js'
import { hexAddress } from '../../../helper/utils.js'
import { testPairsSwitchFunds } from './config.js'
import { Config } from '../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../network/utils.js'

describe.each(testPairsSwitchFunds)(
	'Switch KILTs',
	{ timeout: 30_00000000 },
	async ({ network, storage, accounts, query, sovereignAccount, txContext, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		let receiverAccount: KeyringPair
		const { desc } = config

		// Create the network context
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
		}, 20_0000000)

		// Shut down the network
		afterEach(async () => {
			try {
				await shutDownNetwork([senderContext, receiverContext, relayContext])
			} catch (error) {
				if (!(error instanceof TypeError)) {
					console.error(error)
				}
			}
		})

		it(desc, { timeout: 10_0000000, retry: 0 }, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)

			const { tx, balanceToTransfer } = txContext

			// 1. send native tokens from receiver to sender
			const signedTx1 = tx
				.transfer(receiverContext, hexAddress(receiverAccount.address), balanceToTransfer.foreign.toString())
				.signAsync(receiverAccount)

			const events1 = await sendTransaction(signedTx1)

			// process tx
			await createBlock(receiverContext)
			// process xcm message
			await createBlock(senderContext)

			// 2. send native tokens from sender to receiver

			// Send funds from sender to receiver
			const signedTx = tx
				.switch(senderContext, hexAddress(receiverAccount.address), balanceToTransfer.native.toString())
				.signAsync(senderAccount)

			const events = await sendTransaction(signedTx)
			// process tx
			await createBlock(senderContext)
			// process xcm message
			await createBlock(receiverContext)

			// 3. send native tokens back to sender chain.

			const signedTx3 = tx
				.switchBack(receiverContext, (balanceToTransfer.native / BigInt(2)).toString())
				.signAsync(receiverAccount)

			const events3 = await sendTransaction(signedTx3)
			await createBlock(receiverContext)
			await createBlock(senderContext)

			// 4. send reciever token back

			const signedTx4 = tx
				.withdraw(
					senderContext,
					hexAddress(senderAccount.address),
					(balanceToTransfer.foreign / BigInt(20)).toString()
				)
				.signAsync(senderAccount)

			const events4 = await sendTransaction(signedTx4)
			await createBlock(senderContext)
			await createBlock(receiverContext)

			console.log(senderAccount.address)
			await receiverContext.pause()
		})
	}
)
