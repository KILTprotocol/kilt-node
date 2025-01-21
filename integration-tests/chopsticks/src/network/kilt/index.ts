import * as Main from './main_network.js'
import * as Test from './test_network.js'

export const chains = {
	main: {
		storage: Main.storage,
		getSetupOptions: Main.getSetupOptions,
		parachainInfo: Main.parachainInfo,
	},
	test: {
		storage: Test.storage,
		getSetupOptions: Test.getSetupOptions,
		parachainInfo: Test.parachainInfo,
	},
}
