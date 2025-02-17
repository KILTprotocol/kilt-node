import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../network/utils.js'
import { hexAddress } from '../../../helper/utils.js'
import { testCases } from './config.js'
import type { Config } from '../../../network/types.js'
import { checkSwitchPalletInvariant, getPoolAccount, getReceivedNativeTokens, getRemoteLockedSupply } from '../index.js'
import { spinUpNetwork, tearDownNetwork } from '../../utils.js'

describe.each(testCases)(
	'Switch EKILTs',

	async ({ account, query, txContext, config, sovereignAccount }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc } = config

		// Create the network context
		beforeEach(async () => {
			const { parachainContexts, relayChainContext } = await spinUpNetwork(config)

			relayContext = relayChainContext
			senderContext = parachainContexts[0]
			receiverContext = parachainContexts[1]
			senderAccount = account
		})

		// Shut down the network
		afterEach(async () => {
			await tearDownNetwork([receiverContext, senderContext, relayContext])
		})

		it(desc, { timeout: 10000 }, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)

			const poolAccount = await getPoolAccount(receiverContext)

			// pre condition checks
			const initialRemoteLockedSupply = await getRemoteLockedSupply(receiverContext)
			const initialBalanceReceiverChain = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const initialBalanceSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))
			const initialBalancePoolAccount = await query.receiver(receiverContext, poolAccount)
			const initialBalanceSovereignAccount = await query.sender(senderContext, sovereignAccount.sender)
			const initialBalanceUserSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))
			const initialBalanceUserReceiverChain = await query.receiver(
				receiverContext,
				hexAddress(senderAccount.address)
			)
			expect(initialBalanceReceiverChain).toBe(0n)
			expect(initialBalanceSenderChain).toBeGreaterThan(0n)

			// action
			const { balanceToTransfer, events, tx } = txContext
			const signedTx = tx(senderContext, balanceToTransfer.toString()).signAsync(senderAccount)
			const eventsResult = await sendTransaction(signedTx)

			// process tx
			await createBlock(senderContext)
			// process xcm message
			await createBlock(receiverContext)

			// post condition checks
			const remoteLockedSupply = await getRemoteLockedSupply(receiverContext)
			const balanceSovereignAccount = await query.sender(senderContext, sovereignAccount.sender)
			const balanceSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))
			const balancePoolAccount = await query.receiver(receiverContext, poolAccount)
			const balanceUserReceiverChain = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const receivedFunds = await getReceivedNativeTokens(receiverContext)

			expect(initialBalanceSovereignAccount + balanceToTransfer).toBe(balanceSovereignAccount)
			expect(initialBalanceUserSenderChain - balanceToTransfer).toBe(balanceSenderChain)

			expect(balancePoolAccount + balanceToTransfer).toBe(initialBalancePoolAccount)
			expect(balanceUserReceiverChain - receivedFunds).toBe(initialBalanceUserReceiverChain)

			expect(remoteLockedSupply).toBe(initialRemoteLockedSupply + balanceToTransfer)

			// check events
			await Promise.all(
				events.sender.map(
					async (pallet) =>
						await checkEvents(eventsResult, pallet).toMatchSnapshot(
							`Switch eKILTs sender ${JSON.stringify(pallet)}`
						)
				)
			)

			await Promise.all(
				events.receiver.map((pallet) =>
					checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`Switch eKILTs receiver ${JSON.stringify(pallet)}`
					)
				)
			)

			checkSwitchPalletInvariant(
				expect,
				receiverContext,
				senderContext,
				sovereignAccount.sender,
				query.receiver,
				query.sender
			)
		})
	}
)
