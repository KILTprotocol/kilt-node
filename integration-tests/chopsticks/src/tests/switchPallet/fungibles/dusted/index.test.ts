import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock, setStorage } from '../../../../network/utils.js'
import { calculateTxFees, hexAddress } from '../../../../helper/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../../network/utils.js'

describe.skip.each(testCases)('Dust account', { timeout: 30_000 }, async ({ account, query, txContext, config }) => {
	let context: Config
	let senderAccount: KeyringPair
	let receiverAccount: KeyringPair

	const { desc, network, storage } = config

	// Create the network context
	beforeEach(async () => {
		const { parachains, relay } = network

		const { parachainContexts } = await setupNetwork(relay, parachains)
		const [senderChainContext] = parachainContexts
		context = senderChainContext

		const { senderStorage } = storage
		await setStorage(context, senderStorage)

		senderAccount = account.sender
		receiverAccount = account.receiver
	}, 20_000)

	// Shut down the network
	afterEach(async () => {
		try {
			await shutDownNetwork([context])
		} catch (error) {
			if (!(error instanceof TypeError)) {
				console.error(error)
			}
		}
	})

	it(desc, { timeout: 10_000, retry: 3 }, async ({ expect }) => {
		const { checkSystemEvents } = withExpect(expect)
		const { balanceToTransfer, events, tx } = txContext

		// check initial state

		const nativeBalance = await query.native(context, senderAccount.address)
		const foreignBalance = await query.foreign(context, senderAccount.address)
		const receiverNativeBalance = await query.native(context, receiverAccount.address)

		expect(nativeBalance).toBe(balanceToTransfer)
		expect(foreignBalance).toBeGreaterThan(BigInt(0))
		expect(receiverNativeBalance).toBe(BigInt(0))

		let rawTx = tx(context, hexAddress(receiverAccount.address), balanceToTransfer.toString())
		const txFees = await calculateTxFees(rawTx, senderAccount)
		rawTx = tx(context, hexAddress(receiverAccount.address), (balanceToTransfer - txFees).toString())

		await sendTransaction(rawTx.signAsync(senderAccount))

		// process tx
		await createBlock(context)

		// check balance movement
		const nativeBalanceAfter = await query.native(context, senderAccount.address)
		const foreignBalanceAfter = await query.foreign(context, senderAccount.address)
		const receiverNativeBalanceAfter = await query.native(context, receiverAccount.address)

		expect(nativeBalanceAfter).toBe(BigInt(0))
		// sender should keep the foreign balance
		expect(foreignBalanceAfter).toBeGreaterThan(BigInt(0))
		expect(receiverNativeBalanceAfter).toBe(balanceToTransfer - txFees)

		events.map(
			async (pallet) =>
				await checkSystemEvents(context, pallet).toMatchSnapshot(
					`${desc}: Dusted accounts pallet: ${JSON.stringify(pallet)}`
				)
		)
	})
})
