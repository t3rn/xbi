use xcm::prelude::*;
pub use xcm_executor::traits::Convert as XcmConvert;

// TODO: move me elsewhere, perhaps xbi primitives?
/// A marker trait allowing a multilocation to be converted into an asset id.
pub trait AssetLookup<AssetId: Clone>: XcmConvert<MultiLocation, AssetId> {}

impl<AssetId: Clone> AssetLookup<AssetId> for () {}
