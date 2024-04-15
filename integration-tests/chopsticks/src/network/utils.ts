export function getSiblingDestination(paraId: number) {
	return {
		parents: 1,
		interior: {
			X1: { Parachain: paraId },
		},
	}
}

export function getParentAccountDestination() {
	return {
		parents: 1,
		interior: 'Here',
	}
}

export function getAccountDestinationV2(addr: string) {
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

export function getAccountDestinationV3(addr: string) {
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
