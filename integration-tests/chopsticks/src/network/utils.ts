export function toNumber(value: string | undefined): number | undefined {
	if (value === undefined) {
		return undefined
	}

	return Number(value)
}
