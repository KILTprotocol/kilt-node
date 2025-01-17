process.on('unhandledRejection', (reason, promise) => {
	// Ignore unhandled promise rejections most likely emitted from peer dependencies
	// Optionally log them for debugging if needed:
	console.warn('Unhandled Rejection:', reason, 'Promise:', promise)
})
