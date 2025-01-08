import * as Main from './main_network.js'
import * as Test from './test_network.js'

export default {
	main: {
		storage: Main.storage,
		getSetupOptions: Main.getSetupOptions,
	},
	test: {
		storage: Test.storage,
		getSetupOptions: Test.getSetupOptions,
	},
}
