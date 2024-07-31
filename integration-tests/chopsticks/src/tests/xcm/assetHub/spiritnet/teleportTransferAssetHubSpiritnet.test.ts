import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as SpiritnetConfig from '../../../../network/spiritnet.js'
import * as AssetHubConfig from '../../../../network/assetHub.js'
import { DOT, keysAlice } from '../../../../utils.js'
import { spiritnetContext, assethubContext } from '../../../index.js'
import { getSiblingLocation } from '../../../../network/utils.js'
import { createBlock, hexAddress, setStorage } from '../../../utils.js'

test('Teleport Transfers from AH Account Alice -> Spiritnet Account Alice', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KSM
	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts([keysAlice.address]),
		...AssetHubConfig.assignKSMtoAccounts([keysAlice.address]),
	})

	const spiritnetDestination = { V3: getSiblingLocation(SpiritnetConfig.paraId) }
	const KSMAsset = { V3: [{ id: { Concrete: AssetHubConfig.KSMAssetLocation }, fun: { Fungible: DOT.toString() } }] }
	const remoteFeeId = { V3: { Concrete: AssetHubConfig.KSMAssetLocation } }
	const xcmMessage = {
		V3: [
			{
				DepositAsset: {
					assets: { Wild: 'All' },
					beneficiary: {
						parents: 0,
						interior: {
							X1: {
								AccountId32: {
									id: hexAddress(keysAlice.address),
								},
							},
						},
					},
				},
			},
		],
	}

	// Otherwise the it is tried to route the msg over KSM.
	const signedTx = assethubContext.api.tx.polkadotXcm
		.transferAssetsUsingTypeAndThen(
			spiritnetDestination,
			KSMAsset,
			'Teleport',
			remoteFeeId,
			'Teleport',
			xcmMessage,
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)
	await createBlock(assethubContext)

	// MSG should still be send.
	await checkEvents(events, 'xcmpQueue').toMatchSnapshot(
		`sender assetHub::xcmpQueue::[XcmpMessageSent] asset ${JSON.stringify(KSMAsset)}`
	)
	await checkEvents(events, 'polkadotXcm').toMatchSnapshot(
		`sender assetHub::polkadotXcm::[Attempted,FeesPaid,Sent] asset ${JSON.stringify(KSMAsset)}`
	)
	await checkEvents(events, 'foreignAssets').toMatchSnapshot(
		`sender assetHub::foreignAssets::[Burned] asset ${JSON.stringify(KSMAsset)}`
	)

	// ... But should fail on receiver side.
	await createBlock(spiritnetContext)

	// we expect to have the UntrustedTeleportLocation error
	await checkSystemEvents(spiritnetContext, 'xcmpQueue').toMatchSnapshot(
		`receiver spiritnet::xcmpQueue::[Fail] asset ${JSON.stringify(KSMAsset)}`
	)
}, 20_000)
