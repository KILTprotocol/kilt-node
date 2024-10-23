use substrate_fixed::types::I75F53;

pub type Float = I75F53;

// helper functions
pub fn assert_relative_eq(target: Float, expected: Float, epsilon: Float) {
	assert!(
		(target - expected).abs() <= epsilon,
		"Expected {:?} but got {:?}",
		expected,
		target
	);
}
