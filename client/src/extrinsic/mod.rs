use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};
use substrate_api_client::{
    compose_extrinsic, Api, ExtrinsicParams, Metadata, RpcClient, UncheckedExtrinsicV4,
};

/// Allow us to catch a panicable event and return an option from it.
/// Since substrate_api_client is largely pretty unsafe, we should ensure the macros are caught appropriately.
#[macro_export]
macro_rules! catch_panicable {
    ($tt:expr) => {{
        use std::panic::catch_unwind;
        catch_unwind(|| $tt).ok()
    }};
}

pub mod hrmp;
pub mod sudo;
pub mod xcm;

pub fn index_from_metadata(metadata: Metadata, pallet: String, call: String) -> Option<(u8, u8)> {
    metadata.pallets.get(&pallet).and_then(|pallet| {
        pallet
            .calls
            .get(&call)
            .map(|call_index| (pallet.index, *call_index))
    })
}
