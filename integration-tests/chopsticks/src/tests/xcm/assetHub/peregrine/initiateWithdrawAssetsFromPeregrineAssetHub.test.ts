/* eslint-disable @typescript-eslint/no-unused-vars */
import { test } from 'vitest'
import { sendTransaction, withExpect } from '@acala-network/chopsticks-testing'

import * as PeregrineConfig from '../../../../network/peregrine.js'
import * as AssetHubConfig from '../../../../network/assetHub.js'
import { ROC, initialBalanceKILT, initialBalanceROC, keysAlice, keysBob, keysCharlie } from '../../../../utils.js'
import { peregrineContext, assethubContext, getFreeRocPeregrine, getFreeRocAssetHub } from '../../../index.js'
import { getSiblingLocation } from '../../../../network/utils.js'
import { checkBalance, createBlock, hexAddress, setStorage } from '../../../utils.js'

function getXcmMessage(amount: string | number, beneficiary: string) {
	return {
		V3: [
			{
				WithdrawAsset: [
					{
						id: { Concrete: { parents: 1, interior: 'Here' } },
						fun: { Fungible: amount },
					},
				],
			},
			{
				BuyExecution: {
					fees: {
						id: { Concrete: { parents: 1, interior: 'Here' } },
						fun: { Fungible: amount },
					},
					weightLimit: 'Unlimited',
				},
			},
			{
				InitiateReserveWithdraw: {
					assets: { Wild: 'All' },
					reserve: { parents: 0, interior: 'Here' },
					xcm: [
						{
							BuyExecution: {
								fees: {
									id: { Concrete: { parents: 1, interior: 'Here' } },
									fun: { Fungible: amount },
								},
								weightLimit: 'Unlimited',
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
												network: null,
												id: hexAddress(beneficiary),
											},
										},
									},
								},
							},
						},
					],
				},
			},
		],
	}
}

// TODO: Wait until: https://github.com/KILTprotocol/kilt-node/pull/655

test('Initiate withdraw assets Peregrine Account Alice -> AH Account Bob', async () => {}, 20_000)
