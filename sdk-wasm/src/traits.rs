pub mod blake2 {

	use hash256_std_hasher::Hash256StdHasher;
	use sp_core::{hash::H256, Hasher};
	use sp_core_hashing::blake2_256;

	pub struct BlakeTwo256;

	impl Hasher for BlakeTwo256 {
		type Out = H256;
		type StdHasher = Hash256StdHasher;
		const LENGTH: usize = 32;

		fn hash(x: &[u8]) -> Self::Out {
			blake2_256(x).into()
		}
	}
}
