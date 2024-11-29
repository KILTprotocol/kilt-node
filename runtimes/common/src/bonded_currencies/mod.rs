use substrate_fixed::types::{I75F53, U75F53};

/// The AssetId for bonded assets.
pub type AssetId = u32;

/// Fixed point number used for creating bonding curves.
pub type FloatInput = U75F53;

/// Fixed point number used for doing calculation steps in the bonding curves.
pub type Float = I75F53;
