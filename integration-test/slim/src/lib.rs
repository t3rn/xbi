#![feature(box_syntax)]

use cumulus_parachains_common::{
    impls::NonZeroIssuance, AuraId, MAXIMUM_BLOCK_WEIGHT, SLOT_DURATION,
};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{AsEnsureOriginWithArg, Everything, Nothing, OnTimestampSet},
    weights::{constants::WEIGHT_REF_TIME_PER_SECOND, IdentityFee, Weight},
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::{Id, Sibling};
use polkadot_primitives::v2::Moment;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{ConstU32, ConstU64, Convert, IdentityLookup},
    AccountId32,
};
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowUnpaidExecutionFrom, ConvertedConcreteAssetId, ConvertedConcreteId,
    CurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds, FungiblesAdapter, IsConcrete, LocalMint,
    ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};

pub mod contracts_config;

pub type AccountId = AccountId32;
pub type Balance = u128;
pub type Amount = i128;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();

}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = Everything;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = u64;
    type BlockWeights = ();
    type RuntimeCall = RuntimeCall;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

parameter_types! {
    pub ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl parachain_info::Config for Runtime {}

parameter_types! {
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    pub const RelayNetwork: NetworkId = NetworkId::Rococo;
        pub const SelfLocation: MultiLocation = MultiLocation::here();
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub type LocationToAccountId = (
    ParentIsPreset<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type XcmOriginToCallOrigin = (
    SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
    RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
    SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
    XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
    pub const UnitWeightCost: u64 = 10;
    pub const MaxInstructions: u32 = 100;
}

pub type SovereignAccountOf = (
    SiblingParachainConvertsVia<Id, AccountId>,
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
    pub const AllForLarge: (MultiAssetFilter, MultiLocation) = (MultiAssetFilter::Wild(All), Parachain(3).into_exterior(1));
        pub const MaxAssetsIntoHolding: u32 = 64;

}

pub type TrustedTeleporters = xcm_builder::Case<AllForLarge>;

pub struct XcmConfig;
impl Config for XcmConfig {
    type AssetClaims = ();
    type AssetTransactor = AssetTransactors;
    type AssetTrap = ();
    type Barrier = Barrier;
    type RuntimeCall = RuntimeCall;
    type IsReserve = ();
    type IsTeleporter = TrustedTeleporters;
    type OriginConverter = XcmOriginToCallOrigin;
    type ResponseHandler = ();
    type SubscriptionService = ();
    type Trader = ();
    type AssetLocker = ();
    type UniversalLocation = UniversalLocation;
    type PalletInstancesInfo = AllPalletsWithSystem;
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type FeeManager = ();
    type MessageExporter = ();
    type UniversalAliases = Nothing;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type XcmSender = XcmRouter;
    type AssetExchanger = ();
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.div(4);
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
    type DmpMessageHandler = DmpQueue;
    type RuntimeEvent = RuntimeEvent;
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
    type RuntimeEvent = RuntimeEvent;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type VersionWrapper = ();
    type WeightInfo = ();
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type PriceForSiblingDelivery = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

parameter_types! {
        pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Runtime {
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type RuntimeOrigin = RuntimeOrigin;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmReserveTransferFilter = Everything;
    type XcmRouter = XcmRouter;
    type XcmTeleportFilter = Nothing;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type UniversalLocation = UniversalLocation;
    type TrustedLockers = ();
    type SovereignAccountOf = LocationToAccountId;
    type MaxLockers = ConstU32<8>;
    type WeightInfo = pallet_xcm::TestWeightInfo;

    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account: AccountId) -> MultiLocation {
        X1(Junction::AccountId32 {
            network: Some(NetworkId::Rococo),
            id: account.into(),
        })
        .into()
    }
}

parameter_types! {
    pub const XbiSovereign: AccountId = AccountId32::new([100u8; 32]);
    pub ReserveBalanceCustodian: AccountId = AccountId::new([64u8; 32]);
    pub NotificationWeight: Weight = Weight::from_ref_time(1_000_000_000);
}

impl pallet_xbi_portal::Config for Runtime {
    type AssetRegistry = AssetRegistry;
    type Assets = Assets;
    type RuntimeCall = RuntimeCall;
    type Callback = ();
    type CheckInLimit = ConstU32<100>;
    type CheckInterval = ConstU64<3>;
    type CheckOutLimit = ConstU32<100>;
    type Contracts = ();
    type Currency = Balances;
    type DeFi = ();
    type RuntimeEvent = RuntimeEvent;
    type Evm = Evm;
    type ExpectedBlockTimeMs = ConstU32<6000>;
    type ParachainId = ConstU32<3333>;
    type TimeoutChecksLimit = ConstU32<3000>;
    type Xcm = XcmRouter;
    type XcmSovereignOrigin = XbiSovereign;
    type FeeConversion = IdentityFee<Balance>;
    type ReserveBalanceCustodian = ReserveBalanceCustodian;
    type NotificationWeight = NotificationWeight;
}

pub type AssetId = u32;

parameter_types! {
    pub const AssetDeposit: Balance = 1; // 1 UNIT deposit to create asset
    pub const ApprovalDeposit: Balance = 1;
    pub const AssetsStringLimit: u32 = 50;
    /// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
    // https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
    pub const AssetAccountDeposit: Balance = 0;
}

impl pallet_assets::Config for Runtime {
    type ApprovalDeposit = ApprovalDeposit;
    type AssetAccountDeposit = AssetAccountDeposit;
    type AssetDeposit = AssetDeposit;
    type AssetId = AssetId;
    type Balance = Balance;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type Extra = ();
    type ForceOrigin = EnsureRoot<AccountId>;
    type Freezer = ();
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type StringLimit = AssetsStringLimit;
    type WeightInfo = ();
    type RemoveItemsLimit = ConstU32<1>;
    type AssetIdParameter = AssetId;
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
    type CallbackHandle = ();
}

parameter_types! {
    pub const RegCost: u128 = 100_000_000_000;
}

impl pallet_asset_registry::Config for Runtime {
    type Assets = Assets;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type RegistrationCost = RegCost;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

pub struct MockOnTimestampSet;
impl OnTimestampSet<Moment> for MockOnTimestampSet {
    fn on_timestamp_set(_moment: Moment) {
        // Do nothing
    }
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    // type OnTimestampSet = Aura;
    type OnTimestampSet = MockOnTimestampSet;
    type WeightInfo = ();
}

impl t3rn_primitives::EscrowTrait<Runtime> for Runtime {
    type Currency = Balances;
    type Time = Timestamp;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Config, Event<T>},
        ParachainInfo: parachain_info::{Pallet, Storage, Config},
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
        Aura: pallet_aura,
        Timestamp: pallet_timestamp,
        Assets: pallet_assets,
        AssetRegistry: pallet_asset_registry,
        Evm: pallet_3vm_evm,
        XbiPortal: pallet_xbi_portal = 200
    }
);
