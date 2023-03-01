use xcm::prelude::*;
use xcm_executor::traits::Convert;

// TODO: move me elsewhere, perhaps xbi primitives?
/// A marker trait allowing a multilocation to be converted into an asset id.
pub trait AssetLookup<AssetId: Clone>: Convert<MultiLocation, AssetId> {}

impl<AssetId: Clone> AssetLookup<AssetId> for () {}
