import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../network/peregrine.js'
import * as AssetHubConfig from '../../network/assetHub.js'
import { initialBalanceKILT, initialBalanceROC, keysAlice, keysCharlie } from '../../utils.js'
import {
	peregrineContext,
	getFreeBalancePeregrine,
	getFreeRocPeregrine,
	getFreeEkiltAssetHub,
	assethubContext,
	getFreeRocAssetHub,
	getRemoteLockedSupply,
} from '../index.js'
import { checkBalance, createBlock, setStorage, hexAddress } from '../utils.js'
import { getAccountLocationV3 } from '../../network/utils.js'

test('Switch PILTs against ePILTS on AssetHub', async ({ expect }) => {
	const { checkEvents, checkSystemEvents } = withExpect(expect)

	// Assign alice some KILT and ROC tokens
	await setStorage(peregrineContext, {
		...PeregrineConfig.assignNativeTokensToAccounts([keysAlice.address], initialBalanceKILT),
		...PeregrineConfig.createAndAssignRocs(keysCharlie.address, [keysAlice.address], initialBalanceROC),
		...PeregrineConfig.setSafeXcmVersion3(),
	})

	await setStorage(peregrineContext, PeregrineConfig.setSwitchPair())

	await setStorage(assethubContext, {
		...AssetHubConfig.assignDotTokensToAccounts(
			[keysAlice.address, PeregrineConfig.siblingSovereignAccount],
			initialBalanceROC
		),
		...AssetHubConfig.createForeignAsset(keysCharlie.address, [PeregrineConfig.siblingSovereignAccount]),
	})

	// check initial balance of Alice on Spiritnet
	await checkBalance(getFreeBalancePeregrine, keysAlice.address, expect, initialBalanceKILT)
	await checkBalance(getFreeRocPeregrine, keysAlice.address, expect, initialBalanceROC)

	// Alice should have NO eKILT on AH
	await checkBalance(getFreeEkiltAssetHub, keysAlice.address, expect, BigInt(0))

	// initial balance of the pool account and sovereign account
	const initialBalancePoolAccount = await getFreeBalancePeregrine(PeregrineConfig.initialPoolAccountId)
	const initialBalanceSovereignAccount = await getFreeEkiltAssetHub(PeregrineConfig.siblingSovereignAccount)
	const initialBalanceRocSovereignAccount = await getFreeRocAssetHub(PeregrineConfig.siblingSovereignAccount)
	const initialRemoteLockedSupply = await getRemoteLockedSupply()

	// 50 PILTS
	const balanceToTransfer = BigInt('50000000000000000')

	const beneficiary = getAccountLocationV3(hexAddress(keysAlice.address))

	const signedTx = peregrineContext.api.tx.assetSwitchPool1.switch(balanceToTransfer.toString(), beneficiary)

	const events = await sendTransaction(signedTx.signAsync(keysAlice))

	await createBlock(peregrineContext)

	checkEvents(events, 'xcmpQueue').toMatchSnapshot('sender events xcm queue pallet')
	checkEvents(events, 'assetSwitchPool1').toMatchSnapshot('Switch events assetSwitchPool1 pallet')
	checkEvents(events, { section: 'balances', method: 'Transfer' }).toMatchSnapshot('sender events Balances')

	// check balance. Alice should have less then 50 PILTs
	const freeBalanceAlice = await getFreeBalancePeregrine(keysAlice.address)
	expect(freeBalanceAlice).toBeLessThanOrEqual(balanceToTransfer)

	// check balance Alice. Some fees should have been paid with her rocs:
	const freeRocBalanceAlice = await getFreeRocPeregrine(keysAlice.address)
	expect(freeRocBalanceAlice).eq(initialBalanceROC - BigInt(PeregrineConfig.remoteFee))

	// the Switch pool account should have 50 more PILTs
	const balancePoolAccountAfterTx = await getFreeBalancePeregrine(PeregrineConfig.initialPoolAccountId)
	expect(balancePoolAccountAfterTx).eq(initialBalancePoolAccount + balanceToTransfer)

	await createBlock(assethubContext)
	// Strange behavior here... After creating one block another block with a transfer tx is created. The new block is messing up with the checks. We reset the head here

	const blockNumber = (await assethubContext.api.query.system.number()).toNumber()
	await assethubContext.dev.setHead(blockNumber - 1)

	checkSystemEvents(assethubContext, 'messageQueue').toMatchSnapshot('receiver events messageQueue')
	checkSystemEvents(assethubContext, { section: 'foreignAssets', method: 'Transferred' }).toMatchSnapshot(
		'receiver events Balances'
	)

	// alice should have the exact transferred amount of eKILT. Fees are paid by sovereign account
	const freeBalanceAliceAssetHub = await getFreeEkiltAssetHub(keysAlice.address)
	expect(freeBalanceAliceAssetHub).eq(balanceToTransfer)

	// sovereign account should have less eKILT by the amount of the transferred PILTs
	const freeBalanceSovereignAccount = await getFreeEkiltAssetHub(PeregrineConfig.siblingSovereignAccount)
	expect(initialBalanceSovereignAccount - balanceToTransfer).eq(freeBalanceSovereignAccount)

	// sovereign account should have paid the fees
	const freeRocsSovereignAccount = await getFreeRocAssetHub(PeregrineConfig.siblingSovereignAccount)
	expect(freeRocsSovereignAccount).toBeLessThan(initialBalanceRocSovereignAccount)

	// remote locked supply should have decreased by the amount of the transferred PILTs
	const remoteLockedSupply = await getRemoteLockedSupply()
	expect(remoteLockedSupply).eq(initialRemoteLockedSupply - balanceToTransfer)
}, 20_000)
