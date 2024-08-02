import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import * as RococoConfig from '../../network/rococo.js'
import {
	KILT,
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysCharlie,
} from '../../utils.js'
import { peregrineContext, assethubContext, rococoContext } from '../index.js'
import { createBlock, setStorage, hexAddress, getXcmMessageV4ToSendEkilt } from '../utils.js'
import { getChildLocation, getSiblingLocationV4 } from '../../network/utils.js'

test.skip('Trapped assets', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)
	const switchPairParameters = getAssetSwitchParameters()
	const fundsAlice = switchPairParameters.circulatingSupply

	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [
			keysAlice.address,
			AssetHubConfig.sovereignAccountOnSiblingChains,
		]),
		...PeregrineConfig.setSafeXcmVersion4(),
		...PeregrineConfig.assignNativeTokensToAccounts(
			[keysAlice.address, AssetHubConfig.sovereignAccountOnSiblingChains],
			initialBalanceKILT
		),
		...PeregrineConfig.setSudoKey(keysAlice.address),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair(switchPairParameters))

	await setStorage(rococoContext, {
		...RococoConfig.setSudoKey(keysAlice.address),
		...RococoConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceROC),
	})

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [
			[PeregrineConfig.siblingSovereignAccount, switchPairParameters.sovereignSupply],
			[keysAlice.address, fundsAlice],
		]),
	})

	// First pause switch so that assets are trapped
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.pauseSwitchPair())
		.signAndSend(keysAlice)
	await createBlock(peregrineContext)

	// Now send some funds.
	const balanceToTransfer = fundsAlice / BigInt(2)
	const dest = { v4: getSiblingLocationV4(PeregrineConfig.paraId) }
	const remoteFeeId = { v4: AssetHubConfig.eKiltLocation }

	const funds = {
		v4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: balanceToTransfer.toString() },
			},
		],
	}

	const signedTx = assethubContext.api.tx.polkadotXcm
		.transferAssetsUsingTypeAndThen(
			dest,
			funds,
			'LocalReserve',
			remoteFeeId,
			'LocalReserve',
			getXcmMessageV4ToSendEkilt(keysAlice.address),
			'Unlimited'
		)
		.signAsync(keysAlice)

	const events = await sendTransaction(signedTx)

	await createBlock(assethubContext)

	// msg should be sent.
	checkEvents(events, 'xcmpQueue').toMatchSnapshot(
		`sender AssetHub::xcmpQueue::[XcmpMessageSent] asset ${JSON.stringify(funds)}`
	)
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot(
		`sender AssetHub::polkadotXcm::[Attempted] asset ${JSON.stringify(funds)}`
	)
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		`sender AssetHub::foreignAssets::[Transferred] asset ${JSON.stringify(funds)}`
	)

	await createBlock(peregrineContext)

	// ... But fail on peregrine
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
	await checkSystemEvents(peregrineContext, 'polkadotXcm').toMatchSnapshot(
		'receiver Peregrine::polkadotXcm::[AssetsTrapped]'
	)

	// Resume switch pair again
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.resumeSwitchPair())
		.signAndSend(keysAlice)
	await createBlock(peregrineContext)

	// Alice can reclaim the funds
	const reclaimMsg = [
		{ WithdrawAsset: [{ id: { parents: 0, interior: 'Here' }, fun: { Fungible: KILT } }] },
		{
			BuyExecution: {
				weightLimit: 'Unlimited',
				fees: { id: { parents: 0, interior: 'Here' }, fun: { Fungible: KILT } },
			},
		},
		{
			ClaimAsset: {
				ticket: { parents: 0, interior: { X1: [{ GeneralIndex: 3 }] } },
				assets: [
					{
						id: AssetHubConfig.eKiltLocation,
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
						X1: [
							{
								AccountId32: {
									id: hexAddress(keysAlice.address),
								},
							},
						],
					},
				},
			},
		},
	]

	const peregrineDestination = getSiblingLocationV4(PeregrineConfig.paraId)
	const transactExtrinsic = assethubContext.api.tx.polkadotXcm.send({ V4: peregrineDestination }, { V4: reclaimMsg })
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
	const relayEvents = await sendTransaction(reclaimTx)

	await createBlock(rococoContext)
	await checkEvents(relayEvents, 'sudo').toMatchSnapshot('relayer rococo::sudo::[Sudid]')
	await checkEvents(relayEvents, 'xcmPallet').toMatchSnapshot('relayer rococo::xcmPallet::[Sent]')

	// AH forwards the msg.
	await createBlock(assethubContext)
	await checkSystemEvents(assethubContext, 'messageQueue').toMatchSnapshot(
		'sender AssetHub::messageQueue::[Processed]'
	)
	await checkSystemEvents(assethubContext, 'polkadotXcm').toMatchSnapshot('sender AssetHub::polkadotXcm::[Sent]')
	await checkSystemEvents(assethubContext, 'xcmpQueue').toMatchSnapshot(
		'sender AssetHub::xcmpQueue::[XcmpMessageSent]'
	)

	await createBlock(peregrineContext)
}, 20_00000)
