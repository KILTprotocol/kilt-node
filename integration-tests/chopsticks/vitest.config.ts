import { defineConfig } from 'vite'

export default defineConfig({
	test: {
		maxWorkers: 4,
		minWorkers: 1,
		hideSkippedTests: true,
		// It's not ideal to enable this option, but creating a chopsticks instance for each test
		// can throw an error in the [beforeEach] step, which is resolved in the tests themselves.
		// If the chopsticks instance is not created, the test will fail. By rerunning the test three times,
		// we can ensure that the test will pass, but Vitest will still throw an UnhandledPromiseRejection error,
		// if in any of the retries an error is emitted.
		//If chopsticks is failing to spin up 3 time in a row, the test case will fail.
		dangerouslyIgnoreUnhandledErrors: true,
		retry: 3,
	},
})
