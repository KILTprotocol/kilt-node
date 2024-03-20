import { ApiPromise } from '@polkadot/api'
import { SubmittableExtrinsic } from '@polkadot/api/types'
import type { ISubmittableResult } from '@polkadot/types/types'

export class BlockchainApi {
	api: ApiPromise

	constructor(api: ApiPromise) {
		this.api = api
	}

	queryBalances(api: ApiPromise, address: string) {
		return api.query.system.account(address)
	}

	getLimitedReserveTransfer(amount: any, dest: any, acc: any): SubmittableExtrinsic<'promise', ISubmittableResult> {
		return this.api.tx.polkadotXcm.limitedReserveTransferAssets(
			dest,
			{
				V3: {
					parents: 0,
					interior: {
						X1: {
							AccountId32: {
								id: acc,
							},
						},
					},
				},
			},
			{
				V3: [
					{
						id: { Concrete: { parents: 0, interior: 'Here' } },
						fun: { Fungible: amount },
					},
				],
			},
			0,
			'Unlimited'
		)
	}
}
