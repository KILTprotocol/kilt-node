export function getSiblingLocationV3(paraId: number) {
	return {
		parents: 1,
		interior: {
			X1: { Parachain: paraId },
		},
	}
}

export function getSiblingLocationV4(paraId: number) {
	return {
		V4: {
			parents: 1,
			interior: {
				X1: [{ Parachain: paraId }],
			},
		},
	}
}

export function getChildLocation(paraId: number) {
	return {
		parents: 0,
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

export function getAccountLocationV4(addr: string) {
	return {
		V4: {
			parents: 0,
			interior: {
				X1: [
					{
						AccountId32: {
							id: addr,
						},
					},
				],
			},
		},
	}
}

export function getNativeAssetIdLocationV3(amount: bigint | string) {
	return {
		id: { Concrete: { parents: 0, interior: 'Here' } },
		fun: { Fungible: amount },
	}
}

export function getRelayNativeAssetIdLocationV3(amount: bigint | string) {
	return {
		id: { Concrete: { parents: 1, interior: 'Here' } },
		fun: { Fungible: amount },
	}
}

export function getRelayNativeAssetIdLocationV4(amount: bigint | string) {
	return {
		id: { parents: 1, interior: 'Here' },
		fun: { Fungible: amount },
	}
}
