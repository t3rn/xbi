use super::*;
use crate::assets_config::AssetId;
use cumulus_parachains_common::impls::NonZeroIssuance;
use frame_support::{
    parameter_types,
    traits::{Everything, Nothing, PalletInfoAccess},
    weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::{
    traits::{ConstU32, ConstU64},
    AccountId32,
};
use xcm_builder::{
    AccountId32Aliases, AllowUnpaidExecutionFrom, ConvertedConcreteAssetId, CurrencyAdapter,
    EnsureXcmOrigin, FixedWeightBounds, FungiblesAdapter, IsConcrete, LocationInverter,
    ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};

// Common xcm locations
parameter_types! {
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    // Our representation of the relay asset id
    pub const RelayAssetId: u32 = 1;
    pub const SelfLocation: MultiLocation = MultiLocation::here();
    pub const RelayNetwork: NetworkId = NetworkId::Any;

    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    pub AssetsPalletLocation: MultiLocation =
        PalletInstance(<Assets as PalletInfoAccess>::index() as u8).into();
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

pub type LocationToAccountId = (
    ParentIsPreset<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type XcmOriginToCallOrigin = (
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    XcmPassthrough<Origin>,
);

parameter_types! {
    pub const UnitWeightCost: u64 = 10;
    pub const MaxInstructions: u32 = 100;
}

pub type SovereignAccountOf = (
    SiblingParachainConvertsVia<ParaId, AccountId>,
    AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type LocalAssetTransactor = CurrencyAdapter<
    // Use this currency:
    Balances,
    // Use this currency when it is a fungible asset matching the given location or name:
    IsConcrete<SelfLocation>,
    // We can convert the MultiLocations with our converter above:
    SovereignAccountOf,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // It's a native asset so we keep track of the teleports to maintain total issuance.
    CheckingAccount,
>;

/// Means for transacting assets besides the native currency on this chain.
pub type FungiblesTransactor = FungiblesAdapter<
    Assets,
    // Use the asset registry for lookups
    ConvertedConcreteAssetId<AssetId, Balance, AssetRegistry, JustTry>,
    // Convert an XCM MultiLocation into a local account id:
    LocationToAccountId,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We only want to allow teleports of known assets. We use non-zero issuance as an indication
    // that this asset is known.
    NonZeroIssuance<AccountId, Assets>,
    // The account to use for tracking teleports.
    CheckingAccount,
>;

pub type AssetTransactors = (LocalAssetTransactor, FungiblesTransactor);

/// TODO: this would probably be configured much like the asset registry, e.g basilisk might not allow XCMP but we do.
/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

pub type Barrier = AllowUnpaidExecutionFrom<Everything>;
parameter_types! {
    pub const Roc: MultiAssetFilter = Wild(AllOf { fun: WildFungible, id: Concrete(RelayLocation::get()) });
    pub const AllAssets: MultiAssetFilter = Wild(All);
    pub const RocForRococo: (MultiAssetFilter, MultiLocation) = (Roc::get(), RelayLocation::get());
    pub const RococoForSlim: (MultiAssetFilter, MultiLocation) = (AllAssets::get(), Parachain(1).into()); // Statemine
    pub const RococoForSlender: (MultiAssetFilter, MultiLocation) = (AllAssets::get(), Parachain(2).into());
    pub const RococoForLarge: (MultiAssetFilter, MultiLocation) = (AllAssets::get(), Parachain(3).into());
    pub const RococoForStatemine: (MultiAssetFilter, MultiLocation) = (Roc::get(), Parachain(4).into());
    pub const RococoForCanvas: (MultiAssetFilter, MultiLocation) = (Roc::get(), Parachain(5).into());
    pub const RococoForEncointer: (MultiAssetFilter, MultiLocation) = (Roc::get(), Parachain(6).into());
}

pub type TrustedTeleporters = (
    xcm_builder::Case<RocForRococo>,
    xcm_builder::Case<RococoForSlim>,
    xcm_builder::Case<RococoForSlender>,
    xcm_builder::Case<RococoForLarge>,
    // xcm_builder::Case<RococoForStatemine>,
    // xcm_builder::Case<RococoForCanvas>,
    // xcm_builder::Case<RococoForEncointer>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
    type AssetClaims = ();
    type AssetTransactor = AssetTransactors;
    type AssetTrap = ();
    type Barrier = Barrier;
    type Call = Call;
    type IsReserve = ();
    type IsTeleporter = TrustedTeleporters;
    type LocationInverter = LocationInverter<Ancestry>;
    type OriginConverter = XcmOriginToCallOrigin;
    type ResponseHandler = ();
    type SubscriptionService = ();
    type Trader = ();
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type XcmSender = XcmRouter;
}

parameter_types! {
    pub ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
    pub ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
    type DmpMessageHandler = DmpQueue;
    type Event = Event;
    type OnSystemEvent = ();
    type OutboundXcmpMessageSource = XcmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type SelfParaId = ParachainInfo;
    type XcmpMessageHandler = XcmpQueue;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type ChannelInfo = ParachainSystem;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToCallOrigin;
    type Event = Event;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type VersionWrapper = ();
    type WeightInfo = ();
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Runtime {
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type Call = Call;
    type Event = Event;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type LocationInverter = LocationInverter<Ancestry>;
    type Origin = Origin;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmReserveTransferFilter = Everything;
    type XcmRouter = XcmRouter;
    type XcmTeleportFilter = Nothing;

    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

parameter_types! {
    pub const SelfGatewayId: [u8; 4] = [3, 3, 3, 3];
    pub const XBIAccountId: AccountId = AccountId::new([68u8; 32]); // 0x444...4
    // pub ParachainId: ConstU32 = ConstU32<ParachainInfo::parachain_id().into()>;
    pub const XbiSovereign: AccountId = AccountId32::new([0u8; 32]);
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

impl pallet_xbi_portal::Config for Runtime {
    type AssetRegistry = AssetRegistry;
    type Assets = Assets;
    type Call = Call;
    type Callback = ();
    type CheckInLimit = ConstU32<100>;
    type CheckInterval = ConstU64<3>;
    type CheckOutLimit = ConstU32<100>;
    type Contracts = Contracts;
    type Currency = Balances;
    type DeFi = ();
    type Event = Event;
    type Evm = Evm;
    type ExpectedBlockTimeMs = ConstU32<6000>;
    type ParachainId = ConstU32<3333>;
    type TimeoutChecksLimit = ConstU32<3000>;
    type Xcm = XcmRouter;
    type XcmSovereignOrigin = XbiSovereign;
}
