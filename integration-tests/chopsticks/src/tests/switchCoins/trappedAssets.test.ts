import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assethub.js'
import { KILT, initialBalanceKILT, initialBalanceROC, keysAlice, keysCharlie } from '../../utils.js'
import { peregrineContext, assethubContext } from '../index.js'
import { createBlock, setStorage, hexAddress } from '../utils.js'
import { getSiblingLocation } from '../../network/utils.js'

test('Trapped assets', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSwitchPair(),
		...PeregrineConfig.setSafeXcmVersion3(),
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.setSudoKey(keysAlice.address),
	})

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(
			keysCharlie.address,
			[PeregrineConfig.siblingSovereignAccount, keysAlice.address],
			initialBalanceKILT
		),
	})

	// 50 PILTS
	const balanceToTransfer = initialBalanceKILT / BigInt(2)

	// First pause switch so that assets are trapped
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.pauseSwitchPair())
		.signAndSend(keysAlice)
	await createBlock(peregrineContext)

	const dest = { V3: getSiblingLocation(PeregrineConfig.paraId) }

	const remoteFeeId = { V3: { Concrete: AssetHubConfig.eKiltLocation } }

	const funds = {
		V3: [
			{
				id: { Concrete: AssetHubConfig.eKiltLocation },
				fun: { Fungible: balanceToTransfer },
			},
		],
	}

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

	const signedTx = assethubContext.api.tx.polkadotXcm.transferAssetsUsingTypeAndThen(
		dest,
		funds,
		'LocalReserve',
		remoteFeeId,
		'LocalReserve',
		xcmMessage,
		'Unlimited'
	)

	const events = await sendTransaction(signedTx.signAsync(keysAlice))

	await createBlock(assethubContext)

	// msg should be sent.
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('assetHubs events xcm queue pallet')
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot('PolkadotXcm assethub')
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender events foreignAssets'
	)

	await createBlock(peregrineContext)

	// ... But fail on peregrine
	await checkSystemEvents(peregrineContext, 'xcmpQueue').toMatchSnapshot('peregrine message queue')
	await checkSystemEvents(peregrineContext, 'polkadotXcm').toMatchSnapshot('peregrine asset switch pallet')

	// Resume switch pair again
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.resumeSwitchPair())
		.signAndSend(keysAlice)
	await createBlock(peregrineContext)

	// Alice can reclaim the funds
	const reclaimMsg = [
		{
			ClaimAsset: {
				ticket: getSiblingLocation(AssetHubConfig.paraId),
				assets: [
					{
						id: { Concrete: AssetHubConfig.eKiltLocation },
						// Difficult to say how much funds remain after paying the fees. Let's just say 49 PILTs
						fun: { Fungible: KILT * BigInt(49) },
					},
				],
			},
		},
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
	]

	const innerTx = peregrineContext.api.tx.polkadotXcm.execute({ V3: reclaimMsg }, { refTime: 100, proofSize: 6557 })

	const reclaimTx = peregrineContext.api.tx.sudo
		.sudoUncheckedWeight(innerTx, { refTime: '1000000000000', proofSize: 6557 })
		.signAsync(keysAlice)

	const reclaimEvents = await sendTransaction(reclaimTx)

	await createBlock(peregrineContext)

	await peregrineContext.pause()
}, 20_00000)
