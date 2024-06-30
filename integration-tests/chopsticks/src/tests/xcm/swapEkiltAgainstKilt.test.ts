import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assethub.js'
import { initialBalanceKILT, initialBalanceROC, keysAlice, keysCharlie } from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
	getRemoteLockedSupply,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress } from '../utils.js'
import { getSiblingLocation } from '../../network/utils.js'

test('Swap ePILTs against PILTS on Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([PeregrineConfig.poolAccountId], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSwapPair(),
		...PeregrineConfig.setSafeXcmVersion3(),
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

	// check initial balance of Alice on Spiritnet. Alice should have 0 KILT
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, BigInt(0))
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)

	// Alice should some eKILT on AH
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, initialBalanceKILT)

	// initial balance of the pool account and sovereign account
	const initialBalancePoolAccount = await getFreeBalancePeregrine(PeregrineConfig.poolAccountId)
	const initialBalanceSovereignAccount = await getFreeEkiltAssetHub(PeregrineConfig.siblingSovereignAccount)
	const initialRemoteLockedSupply = await getRemoteLockedSupply()

	// 50 PILTS
	const balanceToTransfer = BigInt('50000000000000000')

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

	// Check sender state
	await createBlock(assethubContext)

	// Check events sender
	checkEvents(events, 'xcmpQueue').toMatchSnapshot('assetHubs events xcm queue pallet')
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot('PolkadotXcm assethub')
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'sender events foreignAssets'
	)

	// check balance. Alice should have less then 50 PILTs
	const freeBalanceAlice = await getFreeEkiltAssetHub(keysAlice.address)
	expect(freeBalanceAlice).toBeLessThanOrEqual(balanceToTransfer)

	// check balance Alice. Some fees should have been paid with her rocs:

	// the sovereign account should have 50 more PILTs
	const balanceSovereignAccountAfterTx = await getFreeEkiltAssetHub(PeregrineConfig.siblingSovereignAccount)
	expect(balanceSovereignAccountAfterTx).eq(initialBalanceSovereignAccount + balanceToTransfer)

	// Check receiver state

	await createBlock(peregrineContext)

	// check events receiver
	checkSystemEvents(peregrineContext, 'xcmpQueue').toMatchSnapshot('peregrine message queue')
	checkSystemEvents(peregrineContext, 'assetSwap').toMatchSnapshot('peregrine asset swap pallet')
	checkSystemEvents(peregrineContext, { section: 'balances', method: 'Transfer' }).toMatchSnapshot(
		'peregrine balances pallet'
	)

	// alice should have some coins now
	const freeBalanceAlicePeregrine = await getFreeBalancePeregrine(keysAlice.address)
	expect(freeBalanceAlicePeregrine).toBeGreaterThan(BigInt(0))

	// Pool account should have less locked PILTs
	const freeBalancePoolAccount = await getFreeBalancePeregrine(PeregrineConfig.poolAccountId)
	// 49 PILTS
	expect(initialBalancePoolAccount - BigInt('49000000000000000')).toBeGreaterThan(freeBalancePoolAccount)

	// remote locked supply should have increased by the amount of the transferred PILTs
	const remoteLockedSupply = await getRemoteLockedSupply()
	expect(remoteLockedSupply).toBeGreaterThan(initialRemoteLockedSupply + BigInt('49000000000000000'))

	//await peregrineContext.pause()
}, 20_000)
