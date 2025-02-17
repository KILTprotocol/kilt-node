import { describe, beforeEach, it, afterEach } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../network/utils.js'
import { calculateTxFees, hexAddress } from '../../../helper/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../network/types.js'
import { checkSwitchPalletInvariant, getPoolAccount, getRemoteLockedSupply } from '../index.js'
import { spinUpNetwork, tearDownNetwork } from '../../utils.js'

describe.each(testCases)(
	'Switch KILTs',

	({ account, query, txContext, config, sovereignAccount }) => {
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

		it(desc, async ({ expect }) => {
			const { checkEvents, checkSystemEvents } = withExpect(expect)

			const poolAccount = await getPoolAccount(senderContext)

			// pre condition checks
			const initialRemoteLockedSupply = await getRemoteLockedSupply(senderContext)
			const initialBalanceReceiverChain = await query.receiver(receiverContext, hexAddress(senderAccount.address))
			const initialBalanceSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))
			const initialBalancePoolAccount = await query.sender(senderContext, poolAccount)
			const initialBalanceUserSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))

			expect(initialBalanceReceiverChain).toBe(0n)
			expect(initialBalanceSenderChain).toBeGreaterThan(0n)

			// action
			const { balanceToTransfer, events, tx } = txContext
			const rawTx = tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString())
			const eventsResult = await sendTransaction(rawTx.signAsync(senderAccount))

			// process tx
			await createBlock(senderContext)
			// process xcm message
			await createBlock(receiverContext)

			// check balance movement
			const txFees = await calculateTxFees(rawTx, senderAccount)
			const remoteLockedSupply = await getRemoteLockedSupply(senderContext)
			const balanceSenderChain = await query.sender(senderContext, hexAddress(senderAccount.address))
			const balancePoolAccount = await query.sender(senderContext, poolAccount)
			const balanceUserReceiverChain = await query.receiver(receiverContext, hexAddress(senderAccount.address))

			// post condition checks
			expect(initialBalanceUserSenderChain - balanceToTransfer - txFees).toBe(balanceSenderChain)
			expect(initialBalancePoolAccount).toBe(balancePoolAccount - balanceToTransfer)
			expect(balanceUserReceiverChain).toBeGreaterThan(0n)
			expect(remoteLockedSupply).toBe(initialRemoteLockedSupply - balanceToTransfer)

			// check events
			await Promise.all(
				events.sender.map((pallet) =>
					checkEvents(eventsResult, pallet).toMatchSnapshot(
						`Switch KILTs sender chain: ${JSON.stringify(pallet)}`
					)
				)
			)

			await Promise.all(
				events.receiver.map((pallet) =>
					checkSystemEvents(receiverContext, pallet).toMatchSnapshot(
						`Switch KILTs receiver chain: ${JSON.stringify(pallet)}`
					)
				)
			)

			// finalize switch
			await createBlock(senderContext)
			await checkSystemEvents(senderContext, 'assetSwitchPool1').toMatchSnapshot('AssetSwitchPool1 Finalization')

			await checkSwitchPalletInvariant(
				expect,
				senderContext,
				receiverContext,
				sovereignAccount.receiver,
				query.sender,
				query.receiver
			)
		})
	}
)
