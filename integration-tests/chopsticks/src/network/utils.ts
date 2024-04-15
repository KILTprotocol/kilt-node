import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

export function getSiblingAccountDestination(paraId: number, addr: string) {
	return {
		parents: 1,
		interior: {
			X2: [
				{ Parachain: paraId },
				{
					AccountId32: {
						id: u8aToHex(decodeAddress(addr)),
					},
				},
			],
		},
	}
}

export function getParentAccountDestination(addr: string) {
	return {
		parents: 1,
		interior: {
			X1: [
				{
					AccountId32: {
						id: u8aToHex(decodeAddress(addr)),
					},
				},
			],
		},
	}
}

export function getNativeAssetIdLocation(amount: bigint) {
	return {
		id: { Concrete: { parents: 0, interior: 'Here' } },
		fun: { Fungible: amount },
	}
}
