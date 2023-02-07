use xcm::prelude::*;
use xcm_executor::traits::Convert;

/// A marker trait allowing a multilocation to be converted into an asset id.
pub trait AssetLookup<AssetId: Clone>: Convert<MultiLocation, AssetId> {}
