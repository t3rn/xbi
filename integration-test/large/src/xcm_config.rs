use super::*;
use crate::assets_config::AssetId;
use cumulus_parachains_common::impls::NonZeroIssuance;
use frame_support::{
    parameter_types,
    traits::{Everything, Nothing, PalletInfoAccess},
    weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_runtime_common::ToAuthor;
use sp_runtime::{
    traits::{ConstU32, ConstU64},
    AccountId32,
};
use xcm_builder::{
    AccountId32Aliases, AllowUnpaidExecutionFrom, ConvertedConcreteAssetId, ConvertedConcreteId,
    CurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds, FungiblesAdapter, IsConcrete, LocalMint,
    NativeAsset, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
    SovereignSignedViaLocation, UsingComponents,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};

// Common xcm locations
parameter_types! {
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    // Our representation of the relay asset id
    pub const RelayAssetId: u32 = 1;
    pub const SelfLocation: MultiLocation = MultiLocation::here();
    pub const RelayNetwork: NetworkId = NetworkId::Rococo;
    pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();

    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
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
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
    pub UnitWeightCost: Weight = Weight::from_parts(10, 0u64);
    pub const MaxInstructions: u32 = 100;
    pub const MaxAssetsIntoHolding: u32 = 64;
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
    (),
>;

/// Means for transacting assets besides the native currency on this chain.
pub type FungiblesTransactor = FungiblesAdapter<
    Assets,
    // Use the asset registry for lookups
    ConvertedConcreteId<
        cumulus_parachains_common::AssetIdForTrustBackedAssets,
        Balance,
        AssetRegistry,
        JustTry,
    >,
    // Convert an XCM MultiLocation into a local account id:
    LocationToAccountId,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We only want to allow teleports of known assets. We use non-zero issuance as an indication
    // that this asset is known.
    LocalMint<cumulus_parachains_common::impls::NonZeroIssuance<AccountId, Assets>>,
    // The account to use for tracking teleports.
    CheckingAccount,
>;

pub type AssetTransactors = (LocalAssetTransactor, FungiblesTransactor);

/// TODO: this would probably be configured much like the asset registry, e.g basilisk might not allow XCMP but we do.
/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

pub type Barrier = AllowUnpaidExecutionFrom<Everything>;
parameter_types! {
    pub const Roc: MultiAssetFilter = Wild(AllOf { fun: WildFungible, id: Concrete(RelayLocation::get()) });
    pub const AllAssets: MultiAssetFilter = Wild(All);
    pub const RocForRococo: (MultiAssetFilter, MultiLocation) = (Roc::get(), RelayLocation::get());
    pub const RococoForSlim: (MultiAssetFilter, MultiLocation) = (AllAssets::get(), Parachain(1).into_location());
    pub const RococoForSlender: (MultiAssetFilter, MultiLocation) = (AllAssets::get(), Parachain(2).into_location());
    pub const RococoForLarge: (MultiAssetFilter, MultiLocation) = (AllAssets::get(), Parachain(3).into_location());
    pub const RococoForStatemine: (MultiAssetFilter, MultiLocation) = (Roc::get(), Parachain(4).into_location());
    pub const RococoForCanvas: (MultiAssetFilter, MultiLocation) = (Roc::get(), Parachain(5).into_location());
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
    type AssetClaims = PolkadotXcm;
    type AssetExchanger = ();
    type AssetLocker = ();
    type AssetTransactor = AssetTransactors;
    // How to withdraw and deposit an asset.
    type AssetTrap = PolkadotXcm;
    type Barrier = Barrier;
    type CallDispatcher = RuntimeCall;
    type FeeManager = ();
    type IsReserve = NativeAsset;
    type IsTeleporter = TrustedTeleporters;
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type MessageExporter = ();
    type OriginConverter = XcmOriginToCallOrigin;
    type PalletInstancesInfo = AllPalletsWithSystem;
    type ResponseHandler = PolkadotXcm;
    type RuntimeCall = RuntimeCall;
    type SafeCallFilter = Everything;
    type SubscriptionService = PolkadotXcm;
    // FIXME: should be using asset_registry
    type Trader = UsingComponents<IdentityFee<Balance>, RelayLocation, AccountId, Balances, ()>;
    type UniversalAliases = Nothing;
    type UniversalLocation = UniversalLocation;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type XcmSender = XcmRouter;
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.div(4);
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
    type DmpMessageHandler = DmpQueue;
    type OnSystemEvent = ();
    type OutboundXcmpMessageSource = XcmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type RuntimeEvent = RuntimeEvent;
    type SelfParaId = ParachainInfo;
    type XcmpMessageHandler = XcmpQueue;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type ChannelInfo = ParachainSystem;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToCallOrigin;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type PriceForSiblingDelivery = ();
    type RuntimeEvent = RuntimeEvent;
    type VersionWrapper = ();
    type WeightInfo = ();
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Runtime {
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    // ^ Override for AdvertisedXcmVersion default
    type Currency = Balances;
    type CurrencyMatcher = ();
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type MaxLockers = ConstU32<8>;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type SovereignAccountOf = LocationToAccountId;
    type TrustedLockers = ();
    type UniversalLocation = UniversalLocation;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type WeightInfo = pallet_xcm::TestWeightInfo;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmReserveTransferFilter = Everything;
    type XcmRouter = XcmRouter;
    type XcmTeleportFilter = Nothing;

    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

parameter_types! {
    pub const XbiSovereign: AccountId = AccountId32::new([104u8; 32]);
    pub ReserveBalanceCustodian: AccountId = AccountId::new([64u8; 32]);
    pub NotificationWeight: Weight = Weight::from_parts(1_000_000_000, 0u64);
}

impl pallet_xbi_portal::Config for Runtime {
    type AssetRegistry = AssetRegistry;
    type Assets = Assets;
    type Callback = ();
    type CheckInLimit = ConstU32<100>;
    type CheckInterval = ConstU64<3>;
    type CheckOutLimit = ConstU32<100>;
    type Contracts = Contracts;
    type Currency = Balances;
    type DeFi = ();
    type Evm = Evm;
    type ExpectedBlockTimeMs = ConstU32<6000>;
    type FeeConversion = IdentityFee<Balance>;
    type NotificationWeight = NotificationWeight;
    type ParachainId = ConstU32<3333>;
    type ReserveBalanceCustodian = ReserveBalanceCustodian;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type TimeoutChecksLimit = ConstU32<3000>;
    type Xcm = XcmRouter;
    type XcmSovereignOrigin = XbiSovereign;
}
