import { describe, beforeEach, it, afterEach } from 'vitest'
import type { KeyringPair } from '@polkadot/keyring/types'
import { sendTransaction } from '@acala-network/chopsticks-testing'
import { u8aToHex } from '@polkadot/util'

import { createBlock, setStorage } from '../../../network/utils.js'
import { testPairsSwapAssets } from './config.js'
import { Config } from '../../../network/types.js'
import { setupNetwork, shutDownNetwork } from '../../../network/utils.js'
import { hexAddress } from '../../../helper/utils.js'

describe.each(testPairsSwapAssets)(
	'Reverse Swap Assets',
	{ timeout: 30_000 },
	async ({ network, storage, accounts, config }) => {
		let senderContext: Config
		let receiverContext: Config
		let relayContext: Config
		let senderAccount: KeyringPair
		const { desc } = config

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

			const { senderAccount: a } = accounts
			senderAccount = a
		}, 30_000)

		afterEach(async () => {
			try {
				await shutDownNetwork([senderContext, receiverContext, relayContext])
			} catch (error) {
				if (!(error instanceof TypeError)) {
					console.error(error)
				}
			}
		})

		it(desc, { timeout: 10_000000, retry: 0 }, async () => {
			const assetKind = {
				parents: 2,
				interior: {
					X2: [
						{ GlobalConsensus: { Ethereum: { chainId: 11155111 } } },
						// Todo: replace with the actual address
						{
							AccountKey20: {
								network: null,
								key: '0x06012c8cf97bead5deae237070f9587f8e7a266d',
							},
						},
					],
				},
			}

			const funds = {
				id: { Concrete: assetKind },
				fun: { Fungible: 8888e9 },
			}

			const dest = {
				V3: {
					parents: 1,
					interior: {
						X1: { Parachain: 2086 },
					},
				},
			}

			console.log(JSON.stringify(funds))

			const remoteFeeId = { V3: { Concrete: assetKind } }

			const tx = senderContext.api.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
				dest,
				{
					V3: [funds],
				},
				'LocalReserve',
				remoteFeeId,
				'LocalReserve',
				{
					V3: [
						{
							DepositAsset: {
								assets: { Wild: 'All' },
								beneficiary: {
									parents: 0,
									interior: {
										X1: {
											AccountId32: {
												id: hexAddress(senderAccount.address),
											},
										},
									},
								},
							},
						},
					],
				},
				'Unlimited'
			)

			const encodedTx1 = u8aToHex(tx.toU8a())

			// returns: 0x8901041f0d0301010099200304000000000b003015661508010300020209079edaa802030006012c8cf97bead5deae237070f9587f8e7a266d0103040d010000010100d150b569a86b95901997e1d98169a7064f486ebdc5272dfcf3d0f2420051c5d300
			console.log(encodedTx1)

			return

			const encodedTx =
				'0x1f0d030101009920030400020209079edaa802030006012c8cf97bead5deae237070f9587f8e7a266d000f0080c3c296931f010300020209079edaa802030006012c8cf97bead5deae237070f9587f8e7a266d0103040d010000010100d150b569a86b95901997e1d98169a7064f486ebdc5272dfcf3d0f2420051c5d300'

			const call = senderContext.api.createType('Call', encodedTx)

			const unsignedTx = senderContext.api.tx(call).signAsync(senderAccount)

			await sendTransaction(unsignedTx)

			await createBlock(senderContext)
			await createBlock(receiverContext)
			await senderContext.pause()

			// check sender state

			console.log('Block created')

			// pallets.sender.map((pallet) =>
			// 	checkEvents(events, pallet).toMatchSnapshot(`sender events ${JSON.stringify(pallet)}`)
			// )

			// const balanceSenderAfterTransfer = await query.sender(senderContext, senderAccount.address)
			// const receiverSovereignAccountBalanceAfterTransfer = await query.sender(
			// 	senderContext,
			// 	sovereignAccount.sender
			// )
			// expect(receiverSovereignAccountBalanceAfterTransfer).toBe(
			// 	receiverSovereignAccountBalanceBeforeTransfer + BigInt(balanceToTransfer)
			// )

			// const removedBalance = balanceToTransfer * BigInt(-1)

			// validateBalanceWithPrecision(
			// 	initialBalanceSender,
			// 	balanceSenderAfterTransfer,
			// 	removedBalance,
			// 	expect,
			// 	precision
			// )

			// // check receiver state
			await createBlock(receiverContext)
			await receiverContext.pause()

			// pallets.receiver.map((pallet) =>
			// 	checkSystemEvents(receiverContext, pallet).toMatchSnapshot(`receiver events ${JSON.stringify(pallet)}`)
			// )

			// const balanceReceiverAfterTransfer = await query.receiver(receiverContext, receiverAccount.address)

			// validateBalanceWithPrecision(
			// 	initialBalanceReceiver,
			// 	balanceReceiverAfterTransfer,
			// 	balanceToTransfer,
			// 	expect,
			// 	precision
			// )
		})
	}
)
