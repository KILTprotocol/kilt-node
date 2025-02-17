import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../network/utils.js'
import { calculateTxFees, getPaidXcmFees, hexAddress } from '../../../helper/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../network/types.js'
import { checkSwitchPalletInvariant } from '../index.js'
import { spinUpNetwork, tearDownNetwork } from '../../utils.js'

describe.each(testCases)(
	'Switch KILTs full flow',

	({ account, query, txContext, config, sovereignAccount }) => {
		let nativeContext: Config
		let foreignContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc } = config

		// Create the network context
		beforeEach(async () => {
			const { parachainContexts, relayChainContext } = await spinUpNetwork(config)
			nativeContext = parachainContexts[0]
			foreignContext = parachainContexts[1]
			relayContext = relayChainContext
			senderAccount = account
		})

		// Shut down the network
		afterEach(async () => {
			await tearDownNetwork([nativeContext, foreignContext, relayContext])
		})

		it(desc, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)
			const { tx, balanceToTransfer, events } = txContext
			const foreignFundsBeforeTx = await query.foreign.nativeFunds(foreignContext, senderAccount.address)

			//action
			// 1. send foreign tokens from foreign chain to native chain
			const txSendForeignAsset = tx.foreign.transfer(
				foreignContext,
				hexAddress(senderAccount.address),
				balanceToTransfer.foreign.toString()
			)

			const events1 = await sendTransaction(txSendForeignAsset.signAsync(senderAccount))

			// process tx
			await createBlock(foreignContext)
			// process xcm message
			await createBlock(nativeContext)

			// check balance movement
			const txFees = await calculateTxFees(txSendForeignAsset, senderAccount)

			const foreignFundsAfterTx = await query.foreign.nativeFunds(foreignContext, senderAccount.address)

			const xcmFees = await getPaidXcmFees(await events1.events)

			expect(foreignFundsBeforeTx - balanceToTransfer.foreign - txFees - xcmFees).toBe(foreignFundsAfterTx)

			// check events
			await Promise.all(
				events.foreign.transfer.map((pallet) =>
					checkEvents(events1, pallet).toMatchSnapshot(
						`transfer foreign funds from foreign chain ${JSON.stringify(pallet)}`
					)
				)
			)

			await Promise.all(
				events.native.receive.foreign.map((pallet) =>
					checkSystemEvents(nativeContext, pallet).toMatchSnapshot(
						`receive foreign funds on native chain ${JSON.stringify(pallet)}`
					)
				)
			)

			// 2. send native tokens
			const nativeBalanceBeforeTx = await query.native.nativeFunds(nativeContext, senderAccount.address)

			// Send funds from native to foreign chainbalanceToTransferBackForeignparaId
			const txSendNativeAsset = tx.native.transfer(
				nativeContext,
				hexAddress(senderAccount.address),
				balanceToTransfer.native.toString()
			)

			const events2 = await sendTransaction(txSendNativeAsset.signAsync(senderAccount))
			// process tx
			await createBlock(nativeContext)
			// process xcm message
			await createBlock(foreignContext)

			// check balance movement

			const txFees2 = await calculateTxFees(txSendNativeAsset, senderAccount)

			const nativeBalanceAfterTx = await query.native.nativeFunds(nativeContext, senderAccount.address)

			expect(nativeBalanceBeforeTx - balanceToTransfer.native - txFees2).toBe(nativeBalanceAfterTx)

			// check events
			await Promise.all(
				events.native.transfer.map((pallet) =>
					checkEvents(events2, pallet).toMatchSnapshot(
						`Transfer native funds to foreign chain ${JSON.stringify(pallet)}`
					)
				)
			)

			await Promise.all(
				events.foreign.receive.native.map((pallet) =>
					checkSystemEvents(foreignContext, pallet).toMatchSnapshot(
						`Receive native funds on foreign chain ${JSON.stringify(pallet)}`
					)
				)
			)

			await checkSwitchPalletInvariant(
				expect,
				nativeContext,
				foreignContext,
				sovereignAccount,
				query.native.nativeFunds,
				query.foreign.foreignFunds
			)

			// 3. send native tokens back to sender chain.
			const balanceToTransferBack = balanceToTransfer.native / 2n

			const nativeBalanceForeignChainBeforeTx = await query.foreign.foreignFunds(
				foreignContext,
				hexAddress(senderAccount.address)
			)

			const signedTx3 = tx.native
				.withdraw(foreignContext, balanceToTransferBack.toString())
				.signAsync(senderAccount)

			const events3 = await sendTransaction(signedTx3)

			// process tx
			await createBlock(foreignContext)
			// process xcm message
			await createBlock(nativeContext)

			// check balance movement

			const nativeBalanceForeignChainAfterTx = await query.foreign.foreignFunds(
				foreignContext,
				senderAccount.address
			)

			expect(nativeBalanceForeignChainBeforeTx - balanceToTransferBack).toBe(nativeBalanceForeignChainAfterTx)

			// check events

			await Promise.all(
				events.foreign.withdraw.map((pallet) =>
					checkEvents(events3, pallet).toMatchSnapshot(
						`Withdraw native funds on foreign chain ${JSON.stringify(pallet)}`
					)
				)
			)

			await Promise.all(
				events.native.receive.native.map((pallet) =>
					checkSystemEvents(nativeContext, pallet).toMatchSnapshot(
						`Receive native funds on native chain ${JSON.stringify(pallet)}`
					)
				)
			)

			// finalize the switch. Create a another block to process the query xcm message
			await createBlock(nativeContext)
			await checkSwitchPalletInvariant(
				expect,
				nativeContext,
				foreignContext,
				sovereignAccount,
				query.native.nativeFunds,
				query.foreign.foreignFunds
			)

			// 4. send foreign token back

			const balanceToTransferBackForeign = balanceToTransfer.foreign / 10n

			const foreignBalanceBeforeTx = await query.native.foreignFunds(nativeContext, senderAccount.address)

			const signedTx4 = tx.foreign
				.withdraw(nativeContext, hexAddress(senderAccount.address), balanceToTransferBackForeign.toString())
				.signAsync(senderAccount)

			const events4 = await sendTransaction(signedTx4)

			// process tx
			await createBlock(nativeContext)

			// process xcm message
			await createBlock(foreignContext)

			// check balance movement
			const foreignBalanceAfterTx = await query.native.foreignFunds(nativeContext, senderAccount.address)
			expect(foreignBalanceBeforeTx - balanceToTransferBackForeign).toBe(foreignBalanceAfterTx)

			// check events

			await Promise.all(
				events.native.withdraw.map((pallet) =>
					checkEvents(events4, pallet).toMatchSnapshot(
						`Withdraw foreign funds on native chain ${JSON.stringify(pallet)}`
					)
				)
			)

			await Promise.all(
				events.foreign.receive.native.map((pallet) =>
					checkSystemEvents(foreignContext, pallet).toMatchSnapshot(
						`Receive foreign funds on foreign chain ${JSON.stringify(pallet)}`
					)
				)
			)
		})
	}
)
