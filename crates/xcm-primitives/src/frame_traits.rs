use xcm::prelude::*;
// pub use xcm_executor::traits::Convert as XcmConvert;

// TODO: move me elsewhere, perhaps xbi primitives?
/// A marker trait allowing a multilocation to be converted into an asset id.
// pub trait AssetLookup<AssetId: Clone>: XcmConvert<MultiLocation, AssetId> {}
pub trait AssetLookup<AssetId: Clone> {
    fn convert_ref(value: impl core::borrow::Borrow<MultiLocation>) -> Result<AssetId, ()>;

    fn reverse_ref(value: impl core::borrow::Borrow<AssetId>) -> Result<MultiLocation, ()>;
}

impl<AssetId: Clone> AssetLookup<AssetId> for () {
    fn convert_ref(value: impl core::borrow::Borrow<MultiLocation>) -> Result<AssetId, ()> {
        Err(())
    }

    fn reverse_ref(value: impl core::borrow::Borrow<AssetId>) -> Result<MultiLocation, ()> {
        Err(())
    }
}
