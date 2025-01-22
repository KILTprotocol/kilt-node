import { describe, beforeEach, it, afterEach } from 'vitest'
import type { KeyringPair } from '@polkadot/keyring/types'
import { setupContext } from '@acala-network/chopsticks-testing'

import { createBlock, setStorage } from '../../../../network/utils.js'
import { testCases } from './config.js'
import { Config } from '../../../../network/types.js'
import { shutDownNetwork } from '../../../../network/utils.js'
import { hexAddress } from '../../../../helper/utils.js'

describe.each(testCases)('Switch KILTs while paused', async ({ account, txContext, config }) => {
	let senderContext: Config
	let senderAccount: KeyringPair
	const { desc, network, storage } = config

	// Create the network context
	beforeEach(async () => {
		const { parachains } = network
		senderContext = await setupContext(parachains[0])

		const { senderStorage } = storage
		await setStorage(senderContext, senderStorage)
		senderAccount = account
	})

	// Shut down the network
	afterEach(async () => {
		try {
			await shutDownNetwork([senderContext])
		} catch (error) {
			if (!(error instanceof TypeError)) {
				console.error(error)
			}
		}
	})

	it(desc, async ({ expect }) => {
		const { balanceToTransfer, tx } = txContext
		let section: string = ''
		let errorName: string = ''

		// This should fail.
		await tx(senderContext, hexAddress(senderAccount.address), balanceToTransfer.toString()).signAndSend(
			senderAccount,
			({ dispatchError }) => {
				if (dispatchError) {
					const decoded = senderContext.api.registry.findMetaError(dispatchError.asModule)
					section = decoded.section
					errorName = decoded.name
				}
			}
		)

		await createBlock(senderContext)

		expect(section).toBe('assetSwitchPool1')
		expect(errorName).toBe('SwitchPairNotEnabled')
	})
})
