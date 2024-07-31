import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import * as RococoConfig from '../../network/rococo.js'
import { KILT, initialBalanceKILT, initialBalanceROC, keysAlice, keysCharlie } from '../../utils.js'
import { peregrineContext, assethubContext, rococoContext } from '../index.js'
import { createBlock, setStorage, hexAddress } from '../utils.js'
import { getChildLocation, getSiblingLocation } from '../../network/utils.js'

test('Trapped assets', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [
			keysAlice.address,
			AssetHubConfig.sovereignAccountOnSiblingChains,
		]),
		...PeregrineConfig.setSwitchPair(),
		...PeregrineConfig.setSafeXcmVersion3(),
		...PeregrineConfig.assignNativeTokensToAccounts(
			[keysAlice.address, AssetHubConfig.sovereignAccountOnSiblingChains],
			initialBalanceKILT
		),
		...PeregrineConfig.setSudoKey(keysAlice.address),
	})

	await setStorage(rococoContext, {
		...RococoConfig.setSudoKey(keysAlice.address),
		...RococoConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceROC),
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
		{ WithdrawAsset: [{ id: { Concrete: { parents: 0, interior: 'Here' } }, fun: { Fungible: KILT } }] },
		{
			BuyExecution: {
				weightLimit: 'Unlimited',
				fees: { id: { Concrete: { parents: 0, interior: 'Here' }, fun: { Fungible: KILT } } },
			},
		},
		{
			ClaimAsset: {
				ticket: { parents: 0, interior: { X1: { GeneralIndex: 3 } } },
				assets: [
					{
						id: { Concrete: AssetHubConfig.eKiltLocation },
						fun: { Fungible: 7161 },
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

	const peregrineDestination = getSiblingLocation(PeregrineConfig.paraId)

	const transactExtrinsic = assethubContext.api.tx.polkadotXcm.send({ V3: peregrineDestination }, { V3: reclaimMsg })

	const assetHubDestination = getChildLocation(AssetHubConfig.paraId)

	const transactMessage = [
		{ UnpaidExecution: { weightLimit: 'Unlimited' } },
		{
			Transact: {
				originKind: 'SuperUser',
				requireWeightAtMost: { refTime: '1000000000', proofSize: '65527' },
				call: {
					encoded: transactExtrinsic.method.toHex(),
				},
			},
		},
	]

	const relayTx = rococoContext.api.tx.xcmPallet.send({ V3: assetHubDestination }, { V3: transactMessage })
	const reclaimTx = rococoContext.api.tx.sudo.sudo(relayTx).signAsync(keysAlice)

	// TODO: check events
	const relayEvents = await sendTransaction(reclaimTx)
	await createBlock(rococoContext)

	await createBlock(assethubContext)

	await createBlock(peregrineContext)

	await peregrineContext.pause()
}, 20_000)
