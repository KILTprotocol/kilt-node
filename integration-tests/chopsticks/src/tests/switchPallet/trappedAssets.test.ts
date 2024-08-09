import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import * as RococoConfig from '../../network/rococo.js'
import {
	KILT,
	ROC,
	getAssetSwitchParameters,
	initialBalanceKILT,
	initialBalanceROC,
	keysAlice,
	keysCharlie,
} from '../../utils.js'
import { peregrineContext, assethubContext, rococoContext, checkSwitchPalletInvariant } from '../index.js'
import { createBlock, setStorage, hexAddress, getXcmMessageV4ToSendEkilt } from '../utils.js'
import { getChildLocation, getSiblingLocationV4 } from '../../network/utils.js'

/**
 * 1. send eKILTs to peregrine while switch is paused
 * 2. enable switch pair again
 * 3. reclaim the assets
 */
test('Trapped assets', async ({ expect }) => {
	const { checkSystemEvents } = withExpect(expect)
	const switchPairParameters = getAssetSwitchParameters()
	const feeAmount = (ROC * BigInt(10)) / BigInt(100)
	const remoteAssetId = { V4: AssetHubConfig.eKiltLocation }
	const remoteXcmFeeId = { V4: { id: AssetHubConfig.nativeTokenLocation, fun: { Fungible: feeAmount } } }
	const remoteReserveLocation = getSiblingLocationV4(AssetHubConfig.paraId)

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

	// pause switch pair
	await setStorage(
		peregrineContext,
		PeregrineConfig.setSwitchPair(
			switchPairParameters,
			remoteAssetId,
			remoteXcmFeeId,
			remoteReserveLocation,
			PeregrineConfig.initialPoolAccountId,
			'Paused'
		)
	)

	await setStorage(rococoContext, {
		...RococoConfig.setSudoKey(keysAlice.address),
		...RococoConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceROC),
	})

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccountsAsStorage(
			[keysAlice.address, PeregrineConfig.sovereignAccountAsSibling],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address),
	})

	await setStorage(
		assethubContext,
		AssetHubConfig.assignForeignAssetToAccounts([
			[PeregrineConfig.sovereignAccountAsSibling, switchPairParameters.sovereignSupply],
			[keysAlice.address, switchPairParameters.circulatingSupply],
		])
	)

	// 1. send the coin and force a trap
	const dest = getSiblingLocationV4(PeregrineConfig.paraId)
	const remoteFeeId = { v4: AssetHubConfig.eKiltLocation }

	const funds = {
		v4: [
			{
				id: AssetHubConfig.eKiltLocation,
				fun: { Fungible: KILT.toString() },
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

	// send msg
	await sendTransaction(signedTx)
	await createBlock(assethubContext)

	// Process msg. Fails on receiver side.
	await createBlock(peregrineContext)
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
	await checkSystemEvents(peregrineContext, 'polkadotXcm').toMatchSnapshot(
		'receiver Peregrine::polkadotXcm::[AssetsTrapped]'
	)

	// 2. enable switch pair again
	await peregrineContext.api.tx.sudo
		.sudo(peregrineContext.api.tx.assetSwitchPool1.resumeSwitchPair())
		.signAndSend(keysAlice)

	//3. reclaim msg
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
				// Specify xcm version 4
				ticket: { parents: 0, interior: { X1: [{ GeneralIndex: 4 }] } },
				assets: [
					{
						id: AssetHubConfig.eKiltLocation,
						fun: { Fungible: KILT.toString() },
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
	const transactExtrinsic = assethubContext.api.tx.polkadotXcm.send(peregrineDestination, { V4: reclaimMsg })
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

	// send msg
	await sendTransaction(reclaimTx)
	await createBlock(rococoContext)

	// forwards the msg.
	await createBlock(assethubContext)

	// Assets should be reclaimed now. Check the events.
	await createBlock(peregrineContext)
	await checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot(
		'receiver Peregrine::messageQueue::[Processed]'
	)
	await checkSystemEvents(peregrineContext, 'polkadotXcm').toMatchSnapshot(
		'receiver Peregrine::polkadotXcm::[AssetsClaimed]'
	)
	await checkSystemEvents(peregrineContext, 'assetSwitchPool1').toMatchSnapshot(
		'receiver Peregrine::assetSwitchPool1::[RemoteToLocalSwitchExecuted]'
	)

	await checkSwitchPalletInvariant(expect)
}, 20_000)
