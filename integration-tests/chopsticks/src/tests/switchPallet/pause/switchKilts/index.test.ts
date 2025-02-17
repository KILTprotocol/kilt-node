import { describe, beforeEach, it, afterEach } from 'vitest'
import type { KeyringPair } from '@polkadot/keyring/types'

import { createBlock } from '../../../../network/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../../network/types.js'
import { hexAddress } from '../../../../helper/utils.js'
import { spinUpNetwork, tearDownNetwork } from '../../../utils.js'

describe.each(testCases)('Switch KILTs while paused', ({ account, txContext, config }) => {
	let senderContext: Config
	let senderAccount: KeyringPair
	const { desc } = config

	// Create the network context
	beforeEach(async () => {
		const { parachainContexts } = await spinUpNetwork(config)

		senderContext = parachainContexts[0]
		senderAccount = account
	})

	// Shut down the network
	afterEach(async () => {
		await tearDownNetwork([senderContext])
	})

	it(desc, async ({ expect }) => {
		const { balanceToTransfer, tx } = txContext
		let error

		// This should fail.
		await tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString()).signAndSend(
			senderAccount,
			({ dispatchError }) => {
				if (dispatchError) {
					error = dispatchError
				}
			}
		)

		await createBlock(senderContext)

		if (!error) {
			throw new Error('Expected SwitchPairNotEnabled error')
		}

		expect(senderContext.api.errors.assetSwitchPool1.SwitchPairNotEnabled.is(error))
	})
})
