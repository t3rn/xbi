use codec::{Decode, Encode};
use sp_runtime::traits::Printable;
use sp_runtime::{DispatchResult, DispatchResultWithInfo};
use std::fmt::Debug;
use xcm::latest::Weight;
use xcm::prelude::*;

/// A shim for routing to PolkadotXcm since it doesnt expose one
pub trait XcmGateway<Origin, Call, PostDispatchInfo>
where
    PostDispatchInfo: Copy + Eq + PartialEq + Encode + Decode + Debug + Printable,
{
    fn send(
        origin: Origin,
        dest: Box<VersionedMultiLocation>,
        message: Box<VersionedXcm<()>>,
    ) -> DispatchResult;
    /// Teleport some assets from the local chain to some destination chain.
    ///
    /// Fee payment on the destination side is made from the asset in the `assets` vector of
    /// index `fee_asset_item`. The weight limit for fees is not provided and thus is unlimited,
    /// with all fees taken as needed from the asset.
    ///
    /// - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
    /// - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))` to send
    ///   from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
    /// - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will generally be
    ///   an `AccountId32` value.
    /// - `assets`: The assets to be withdrawn. The first item should be the currency used to to pay the fee on the
    ///   `dest` side. May not be empty.
    /// - `fee_asset_item`: The index into `assets` of the item which should be used to pay
    ///   fees.
    fn teleport_assets(
        origin: Origin,
        dest: Box<VersionedMultiLocation>,
        beneficiary: Box<VersionedMultiLocation>,
        assets: Box<VersionedMultiAssets>,
        fee_asset_item: u32,
    ) -> DispatchResult;

    /// Transfer some assets from the local chain to the sovereign account of a destination
    /// chain and forward a notification XCM.
    ///
    /// Fee payment on the destination side is made from the asset in the `assets` vector of
    /// index `fee_asset_item`. The weight limit for fees is not provided and thus is unlimited,
    /// with all fees taken as needed from the asset.
    ///
    /// - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
    /// - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))` to send
    ///   from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
    /// - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will generally be
    ///   an `AccountId32` value.
    /// - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on the
    ///   `dest` side.
    /// - `fee_asset_item`: The index into `assets` of the item which should be used to pay
    ///   fees.
    fn reserve_transfer_assets(
        origin: Origin,
        dest: Box<VersionedMultiLocation>,
        beneficiary: Box<VersionedMultiLocation>,
        assets: Box<VersionedMultiAssets>,
        fee_asset_item: u32,
    ) -> DispatchResult;

    /// Execute an XCM message from a local, signed, origin.
    ///
    /// An event is deposited indicating whether `msg` could be executed completely or only
    /// partially.
    ///
    /// No more than `max_weight` will be used in its attempted execution. If this is less than the
    /// maximum amount of weight that the message could take to be executed, then no execution
    /// attempt will be made.
    ///
    /// NOTE: A successful return to this does *not* imply that the `msg` was executed successfully
    /// to completion; only that *some* of it was executed.
    fn execute(
        origin: Origin,
        message: Box<VersionedXcm<Call>>,
        max_weight: Weight,
    ) -> DispatchResultWithInfo<PostDispatchInfo>;

    /// Extoll that a particular destination can be communicated with through a particular
    /// version of XCM.
    ///
    /// - `origin`: Must be Root.
    /// - `location`: The destination that is being described.
    /// - `xcm_version`: The latest version of XCM that `location` supports.
    fn force_xcm_version(
        origin: Origin,
        location: Box<MultiLocation>,
        xcm_version: XcmVersion,
    ) -> DispatchResult;

    /// Set a safe XCM version (the version that XCM should be encoded with if the most recent
    /// version a destination can accept is unknown).
    ///
    /// - `origin`: Must be Root.
    /// - `maybe_xcm_version`: The default XCM encoding version, or `None` to disable.
    fn force_default_xcm_version(
        origin: Origin,
        maybe_xcm_version: Option<XcmVersion>,
    ) -> DispatchResult;

    /// Ask a location to notify us regarding their XCM version and any changes to it.
    ///
    /// - `origin`: Must be Root.
    /// - `location`: The location to which we should subscribe for XCM version notifications.
    fn force_subscribe_version_notify(
        origin: Origin,
        location: Box<VersionedMultiLocation>,
    ) -> DispatchResult;

    /// Require that a particular destination should no longer notify us regarding any XCM
    /// version changes.
    ///
    /// - `origin`: Must be Root.
    /// - `location`: The location to which we are currently subscribed for XCM version
    ///   notifications which we no longer desire.
    fn force_unsubscribe_version_notify(
        origin: Origin,
        location: Box<VersionedMultiLocation>,
    ) -> DispatchResult;

    /// Transfer some assets from the local chain to the sovereign account of a destination
    /// chain and forward a notification XCM.
    ///
    /// Fee payment on the destination side is made from the asset in the `assets` vector of
    /// index `fee_asset_item`, up to enough to pay for `weight_limit` of weight. If more weight
    /// is needed than `weight_limit`, then the operation will fail and the assets send may be
    /// at risk.
    ///
    /// - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
    /// - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))` to send
    ///   from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
    /// - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will generally be
    ///   an `AccountId32` value.
    /// - `assets`: The assets to be withdrawn. This should include the assets used to pay the fee on the
    ///   `dest` side.
    /// - `fee_asset_item`: The index into `assets` of the item which should be used to pay
    ///   fees.
    /// - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
    fn limited_reserve_transfer_assets(
        origin: Origin,
        dest: Box<VersionedMultiLocation>,
        beneficiary: Box<VersionedMultiLocation>,
        assets: Box<VersionedMultiAssets>,
        fee_asset_item: u32,
        weight_limit: WeightLimit,
    ) -> DispatchResult;

    /// Teleport some assets from the local chain to some destination chain.
    ///
    /// Fee payment on the destination side is made from the asset in the `assets` vector of
    /// index `fee_asset_item`, up to enough to pay for `weight_limit` of weight. If more weight
    /// is needed than `weight_limit`, then the operation will fail and the assets send may be
    /// at risk.
    ///
    /// - `origin`: Must be capable of withdrawing the `assets` and executing XCM.
    /// - `dest`: Destination context for the assets. Will typically be `X2(Parent, Parachain(..))` to send
    ///   from parachain to parachain, or `X1(Parachain(..))` to send from relay to parachain.
    /// - `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will generally be
    ///   an `AccountId32` value.
    /// - `assets`: The assets to be withdrawn. The first item should be the currency used to to pay the fee on the
    ///   `dest` side. May not be empty.
    /// - `fee_asset_item`: The index into `assets` of the item which should be used to pay
    ///   fees.
    /// - `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase.
    fn limited_teleport_assets(
        origin: Origin,
        dest: Box<VersionedMultiLocation>,
        beneficiary: Box<VersionedMultiLocation>,
        assets: Box<VersionedMultiAssets>,
        fee_asset_item: u32,
        weight_limit: WeightLimit,
    ) -> DispatchResult;
}
