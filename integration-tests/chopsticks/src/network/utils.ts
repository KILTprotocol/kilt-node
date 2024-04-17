export function getSiblingLocation(paraId: number) {
	return {
		parents: 1,
		interior: {
			X1: { Parachain: paraId },
		},
	}
}

export function getParentLocation() {
	return {
		parents: 1,
		interior: 'Here',
	}
}

export function getAccountLocationV2(addr: string) {
	return {
		V2: {
			parents: 0,
			interior: {
				X1: {
					AccountId32: {
						network: 'Any',
						id: addr,
					},
				},
			},
		},
	}
}

export function getAccountLocationV3(addr: string) {
	return {
		V3: {
			parents: 0,
			interior: {
				X1: {
					AccountId32: {
						id: addr,
					},
				},
			},
		},
	}
}

export function getNativeAssetIdLocation(amount: bigint) {
	return {
		id: { Concrete: { parents: 0, interior: 'Here' } },
		fun: { Fungible: amount },
	}
}
