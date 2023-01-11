use xcm::prelude::*;
use xcm_executor::traits::Convert;

pub trait AssetLookup<AssetId: Clone>: Convert<MultiLocation, AssetId> {}
