import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import { getAssetSwitchParameters, initialBalanceKILT, initialBalanceROC, keysAlice, keysCharlie } from '../../utils.js'
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

test('Switch ePILTs against PILTS on Peregrine', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	await setStorage(peregrineContext, {
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSwitchPair(getAssetSwitchParameters()),
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

	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, initialBalanceKILT)

	const initialBalancePoolAccount = await getFreeBalancePeregrine(PeregrineConfig.initialPoolAccountId)
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
				fun: { Fungible: balanceToTransfer.toString() },
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

	checkEvents(events, 'xcmpQueue').toMatchSnapshot(
		`sender AssetHubs::xcmpQueue::[XcmpMessageSent] ${JSON.stringify(funds)}`
	)
	checkEvents(events, { section: 'polkadotXcm', method: 'Attempted' }).toMatchSnapshot(
		`sender AssetHub::polkadotXcm::[Attempted] ${JSON.stringify(funds)}`
	)
	checkEvents(events, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		`sender AssetHub::foreignAssets::[Transferred] ${JSON.stringify(funds)}`
	)

	// check balance. Alice should have >= 50 PILTs
	const freeBalanceAlice = await getFreeEkiltAssetHub(keysAlice.address)
	expect(freeBalanceAlice).toBeLessThanOrEqual(balanceToTransfer)

	// the sovereign account should have 50 more PILTs
	const balanceSovereignAccountAfterTx = await getFreeEkiltAssetHub(PeregrineConfig.siblingSovereignAccount)
	expect(balanceSovereignAccountAfterTx).eq(initialBalanceSovereignAccount + balanceToTransfer)

	await createBlock(peregrineContext)

	checkSystemEvents(peregrineContext, 'messageQueue').toMatchSnapshot('receiver Peregrine::messageQueue::[Processed]')
	checkSystemEvents(peregrineContext, 'assetSwitchPool1').toMatchSnapshot(
		'receiver Peregrine::assetSwitchPool1::[RemoteToLocalSwitchExecuted]'
	)
	checkSystemEvents(peregrineContext, { section: 'balances', method: 'Transfer' }).toMatchSnapshot(
		'receiver Peregrine::balances::[Transfer]'
	)

	// alice should have some coins now
	const freeBalanceAlicePeregrine = await getFreeBalancePeregrine(keysAlice.address)
	expect(freeBalanceAlicePeregrine).toBeGreaterThan(BigInt(0))

	// Pool account should have less locked PILTs
	const freeBalancePoolAccount = await getFreeBalancePeregrine(PeregrineConfig.initialPoolAccountId)
	expect(initialBalancePoolAccount - balanceToTransfer).toBeGreaterThanOrEqual(freeBalancePoolAccount)

	// remote locked supply should have increased by the amount of the transferred PILTs
	const remoteLockedSupply = await getRemoteLockedSupply()
	expect(remoteLockedSupply).toBeGreaterThanOrEqual(initialRemoteLockedSupply + balanceToTransfer)
}, 20_000)
