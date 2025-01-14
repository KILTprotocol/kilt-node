import { defineConfig } from 'vite'

export default defineConfig({
	test: {
		maxWorkers: 4,
		minWorkers: 1,
		hideSkippedTests: true,
	},
})
