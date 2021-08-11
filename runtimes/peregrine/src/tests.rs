use super::Call;

#[test]
fn call_size() {
	assert!(
		core::mem::size_of::<Call>() <= 230,
		"size of Call is {:?} bytes which is more than 230 bytes: some calls have too big arguments, use Box to reduce the size of Call.
		If the limit is too strong, maybe consider increase the limit to 300.",
		core::mem::size_of::<Call>(),
	);
}
