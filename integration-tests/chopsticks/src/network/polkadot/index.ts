import { Chains } from '../types.js'
import * as Main from './main_network.js'
import * as Test from './test_network.js'

export const chains: Chains = {
	main: {
		storage: Main.storage,
		getConfig: Main.getSetupOptions,
		chainInfo: {},
	},
	test: {
		storage: Test.storage,
		getConfig: Test.getSetupOptions,
		chainInfo: {},
	},
}
