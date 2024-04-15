import { u8aToHex } from '@polkadot/util'
import { decodeAddress } from '@polkadot/util-crypto'

export function getSiblingAccountDestinationV3(paraId: number, addr: string) {
	return {
		V3: {
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
		},
	}
}

export function getSiblingAccountDestinationV2(paraId: number, addr: string) {
	return {
		V2: {
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
		},
	}
}

export function getParentAccountDestinationV3(addr: string) {
	return {
		V3: {
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
		},
	}
}

export function getParentAccountDestinationV2(addr: string) {
	return {
		V2: {
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
		},
	}
}
