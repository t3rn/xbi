#![cfg_attr(not(feature = "std"), no_std)]
#![feature(result_option_inspect)]
#![allow(clippy::type_complexity)]

pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::DispatchResult,
    weights::{Weight, WeightToFee},
};
use frame_system::pallet_prelude::OriginFor;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{CheckedMul, Zero},
    DispatchError, Either,
};
use sp_std::{marker::PhantomData, prelude::*};
use xcm_executor::{
    traits::{ShouldExecute, WeightTrader},
    Assets,
};

use xcm::latest::Weight as XCMWeight;

pub mod convert;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// TODO
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

t3rn_primitives::reexport_currency_types!();
t3rn_primitives::reexport_asset_types!();

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct AssetInfo<AssetId, AccountId, Balance> {
    id: AssetId,
    /// A set of capabilities an asset has
    capabilities: Vec<Capability<AccountId, Balance>>,
    /// The "official" location for the asset, used in reverse lookups
    location: MultiLocation,
}

impl<AssetId, AccountId, Balance> AssetInfo<AssetId, AccountId, Balance> {
    pub fn new(
        id: AssetId,
        location: MultiLocation,
        capabilities: Vec<Capability<AccountId, Balance>>,
    ) -> Self {
        Self {
            id,
            location,
            capabilities,
        }
    }
}

#[derive(Encode, Decode, Debug, Copy, Clone, PartialEq, Eq, TypeInfo)]
pub enum Capability<AccountId, Balance> {
    /// Can the asset be teleported, if so, what is the checking account for it?
    Teleport(Option<AccountId>),
    /// Can the asset be used as a reserve, if so, what is the checking account for it?
    Reserve(Option<AccountId>),
    /// Can the asset be used as payment, if so, what is the weight for it?
    Payable { fees_per_weight: Option<Balance> },
}

// TODO[Style]: needs refactor into Index
/// The amount of capabilities we have
pub const CAPABILITY_COUNT: usize = 3;

impl<AccountId, Balance> Capability<AccountId, Balance> {
    // I was not able to get this to work dynamically
    pub fn as_usize(&self) -> usize {
        match self {
            Capability::Teleport(_) => 0,
            Capability::Reserve(_) => 1,
            Capability::Payable { .. } => 2,
        }
    }

    fn has_value(&self) -> bool {
        match self {
            Capability::Teleport(arg) | Capability::Reserve(arg) => arg.is_some(),
            Capability::Payable {
                fees_per_weight: arg,
            } => arg.is_some(),
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    pub use frame_support::sp_runtime::Either;
    use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
    use frame_system::pallet_prelude::*;
    use sp_std::vec;
    pub use xcm::prelude::*;

    use crate::{
        can_put_capabilities, soft_capability_lookup, strict_capability_lookup, AssetIdOf,
        AssetInfo, BalanceOf, Capability, Vec,
    };

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Call: From<Call<Self>>;

        type Currency: ReservableCurrency<Self::AccountId>;

        type Assets: frame_support::traits::tokens::fungibles::Inspect<Self::AccountId>;

        type RegistrationCost: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn location_mapping)]
    pub type LocationMapping<T> =
        StorageMap<_, Blake2_128, MultiLocation, AssetIdOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn asset_metadata)]
    pub type AssetMetadata<T> = StorageMap<
        _,
        Blake2_128,
        AssetIdOf<T>,
        crate::AssetInfo<AssetIdOf<T>, <T as frame_system::Config>::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An asset location mapping was registered. [id, location]
        Registered {
            asset_id: AssetIdOf<T>,
            location: MultiLocation,
        },
        /// An asset's information was created or updated [id, location]
        Info {
            asset_id: AssetIdOf<T>,
            location: MultiLocation,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// The mapping or asset could not be found
        NotFound,
        /// This location mapping was unallowed for this user
        LocationUnallowed,
        /// One of the passed capabilities is not valid for this asset
        CapabilitiesNotPermitted,
        /// The XCM message shouldnt be executed for given asset
        ShouldntExecuteMessage,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// A dispatchable that allows anyone to register a mapping for an asset
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register(
            origin: OriginFor<T>,
            location: MultiLocation,
            id: AssetIdOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed_or_root(origin)?;

            let is_parent = location == MultiLocation::parent();
            let is_self = location == MultiLocation::here();

            let is_parachain = location
                .interior()
                .iter()
                .any(|j| matches!(j, Parachain(_) | &PalletInstance(_)));
            let can_register = !is_parent && !is_self && !is_parachain;

            let is_root = who.is_none();
            // Root can register anything
            if is_root || can_register {
                <LocationMapping<T>>::insert(location.clone(), id);
                Self::deposit_event(Event::Registered {
                    asset_id: id,
                    location,
                });
                Ok(())
            } else {
                Err(Error::<T>::LocationUnallowed.into())
            }
        }

        // TODO: expand this to allow over XBI
        /// A dispatchable that allows sudo to register asset information
        /// In the future this can be updated either by owners, parachains over xcm or by sudo
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_info(
            origin: OriginFor<T>,
            info: AssetInfo<AssetIdOf<T>, T::AccountId, BalanceOf<T>>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            can_put_capabilities::<T>(&info.capabilities)?;

            <AssetMetadata<T>>::insert(info.id, info.clone());
            Self::deposit_event(Event::Info {
                asset_id: info.id,
                location: info.location,
            });
            Ok(())
        }
    }

    /// A function evaluating if given capabilities are permitted for an asset
    pub fn check_capabilities<T: Config>(
        id: Either<AssetIdOf<T>, MultiLocation>,
        capabilities: Vec<Capability<T::AccountId, BalanceOf<T>>>,
    ) -> Result<Vec<Capability<T::AccountId, BalanceOf<T>>>, DispatchError> {
        log::debug!(target: "asset-registry", "Checking capabilities for {:?}, capabilities: {:?}", id, capabilities);
        let asset_id = match id {
            Either::Left(id) => id,
            Either::Right(location) => <LocationMapping<T>>::get(location)
                .ok_or(Error::<T>::NotFound)
                .inspect_err(
                    |_| log::debug!(target: "asset-registry", "Location mapping not found"),
                )?,
        };

        if let Some(info) = <Pallet<T>>::asset_metadata(asset_id) {
            log::debug!(target: "asset-registry", "Found asset info: {:?}", info);
            let mut result: Vec<Capability<T::AccountId, BalanceOf<T>>> = vec![];
            for capability in capabilities {
                result.push(match capability.has_value() {
                    true => strict_capability_lookup::<T>(capability, &info.capabilities),
                    false => soft_capability_lookup::<T>(&capability, &info.capabilities),
                }?)
            }
            Ok(result)
        } else {
            log::debug!(target: "asset-registry", "Asset info not found");
            Err(Error::<T>::NotFound.into())
        }
    }
}

pub trait AssetRegistry<Origin, AccountId, Balance, AssetId> {
    fn register(origin: Origin, location: MultiLocation, id: AssetId) -> DispatchResult;

    fn register_info(
        origin: Origin,
        info: AssetInfo<AssetId, AccountId, Balance>,
    ) -> DispatchResult;

    fn lookup(
        key: Either<MultiLocation, AssetId>,
    ) -> Result<Either<AssetId, MultiLocation>, DispatchError>;

    fn check_capabilities(
        id: Either<AssetId, MultiLocation>,
        capabilities: Vec<Capability<AccountId, Balance>>,
    ) -> Result<Vec<Capability<AccountId, Balance>>, DispatchError>;
}

impl<T: Config> AssetRegistry<OriginFor<T>, T::AccountId, BalanceOf<T>, AssetIdOf<T>>
    for Pallet<T>
{
    fn register(origin: OriginFor<T>, location: MultiLocation, id: AssetIdOf<T>) -> DispatchResult {
        <Pallet<T>>::register(origin, location, id)
    }

    fn register_info(
        origin: OriginFor<T>,
        info: AssetInfo<AssetIdOf<T>, T::AccountId, BalanceOf<T>>,
    ) -> DispatchResult {
        <Pallet<T>>::register_info(origin, info)
    }

    fn lookup(
        key: Either<MultiLocation, AssetIdOf<T>>,
    ) -> Result<Either<AssetIdOf<T>, MultiLocation>, DispatchError> {
        log::debug!(target: "asset-registry", "Lookup for key({:?})", key);

        match key {
            Either::Left(location) => Ok(Either::Left(
                <LocationMapping<T>>::get(location)
                    .ok_or(Error::<T>::NotFound)
                    .inspect_err(
                        |e| log::debug!(target: "asset-registry", "Lookup failed {:?}", e),
                    )?,
            )),
            Either::Right(asset_id) => Ok(Either::Right(
                <AssetMetadata<T>>::get(asset_id)
                    .ok_or(Error::<T>::NotFound)
                    .inspect_err(
                        |e| log::debug!(target: "asset-registry", "Lookup failed {:?}", e),
                    )?
                    .location,
            )),
        }
        .inspect(|x| log::debug!(target: "asset-registry", "Found value for lookup: {:?}", x))
    }

    fn check_capabilities(
        id: Either<AssetIdOf<T>, MultiLocation>,
        capabilities: Vec<Capability<T::AccountId, BalanceOf<T>>>,
    ) -> Result<Vec<Capability<T::AccountId, BalanceOf<T>>>, DispatchError> {
        crate::check_capabilities::<T>(id, capabilities)
    }
}

impl<T: Config> ShouldExecute for Pallet<T> {
    fn should_execute<Call>(
        origin: &MultiLocation,
        message: &mut Xcm<Call>,
        _max_weight: Weight,
        _weight_credit: &mut Weight,
    ) -> Result<(), ()> {
        log::debug!(target: "asset-registry", "Should execute for origin({:?}) and message({:?})", origin, message);
        // first, get ID from location
        let id = <Pallet<T>>::lookup(Either::Left(origin.clone()))
            .map_err(|_| ())?
            .left()
            .ok_or(())
            .inspect_err(|_| log::debug!(target: "asset-registry", "ShouldntExecuteMessage - Asset lookup not found"))?;

        // get info from ID
        let info = <AssetMetadata<T>>::get(id).ok_or(()).inspect_err(
            |_| log::debug!(target: "asset-registry", "ShouldntExecuteMessage - Asset not found"),
        )?;

        let mut has_checked = [false; CAPABILITY_COUNT];

        //ensure the capabilities are permitted for given asset
        let (_, errors) = message
            .0
            .iter()
            .filter_map(|i| Capability::<T::AccountId, BalanceOf<T>>::try_from(i).ok())
            .map(|capability: Capability<T::AccountId, BalanceOf<T>>| {
                if !has_checked[capability.as_usize()] {
                    has_checked[capability.as_usize()] = true;
                    soft_capability_lookup::<T>(&capability, &info.capabilities).map(|_| ()).inspect_err(|_| {
                        log::debug!(target: "asset-registry", "ShouldntExecuteMessage - Capability not permitted: {:?}", capability);
                    })
                } else {
                    Ok(())
                }
            })
            .partition::<Vec<_>, _>(|x| x.is_ok());

        if errors.is_empty() {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub struct WeightAssetConvert<T: pallet::Config, WeightToFeeConverter> {
    _phantom: PhantomData<(T, WeightToFeeConverter)>,
    weight: Weight,
    location: Option<MultiLocation>,
    fee_per_weight: BalanceOf<T>,
}

impl<T: Config, WeightToFeeConverter: WeightToFee<Balance = BalanceOf<T>>> WeightTrader
    for WeightAssetConvert<T, WeightToFeeConverter>
{
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
            weight: Zero::zero(),
            location: None,
            fee_per_weight: Zero::zero(),
        }
    }

    fn buy_weight(&mut self, weight: XCMWeight, payment: Assets) -> Result<Assets, XcmError> {
        log::trace!(target: "xcm::weight", "TakeFirstAssetTrader::buy_weight weight: {:?}, payment: {:?}", weight, payment);

        // We take the very first multiasset from payment, sorted by .into()
        let multiassets: MultiAssets = payment.clone().into();
        // Take the first multiasset from the selected MultiAssets
        let first_asset = multiassets.get(0).ok_or(XcmError::AssetNotFound)?;

        let asset_id = if let Concrete(location) = &first_asset.id {
            self.location = Some(location.clone());
            <LocationMapping<T>>::get::<MultiLocation>(location.clone())
                .ok_or(XcmError::AssetNotFound)?
        } else {
            return Err(XcmError::AssetNotFound);
        };

        // Get metadata for first asset from storages
        let info = <AssetMetadata<T>>::get(asset_id).ok_or(XcmError::AssetNotFound)?;

        let payable = soft_capability_lookup::<T>(
            &Capability::Payable {
                fees_per_weight: None,
            },
            &info.capabilities,
        )
        .map_err(|_| XcmError::WeightNotComputable)?;

        let fee = WeightToFeeConverter::weight_to_fee(&weight);

        // compute payment amount based on rate found in storage
        let payable_amount: u128 = match payable {
            Capability::Payable {
                fees_per_weight: Some(value),
            } => {
                // One thing to consider here is that this might be abitragable during fee_per_weight changes.
                self.fee_per_weight = value;
                fee.checked_mul(&value)
                    .ok_or(XcmError::WeightNotComputable)?
                    .try_into()
                    .map_err(|_| XcmError::WeightNotComputable)?
            }
            _ => return Err(XcmError::WeightNotComputable),
        };

        let unused = payment
            .checked_sub((first_asset.id.clone(), payable_amount).into())
            .map_err(|_| XcmError::TooExpensive)?;

        // Assign after all checks have passed
        self.weight = self.weight.saturating_add(weight);

        Ok(unused)
    }

    fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
        log::trace!(
            target: "xcm::weight", "MultiCurrencyTrader::refund_weight weight: {:?}, paid_assets: {:?}",
            weight, self.location
        );

        // ensure weight <= self.weight, which is the amount that was bought
        let weight = weight.min(self.weight);

        if weight <= Zero::zero() {
            return None; // return if no weight can be refunded
        }

        let fee = WeightToFeeConverter::weight_to_fee(&weight);

        if let Some(location) = &self.location {
            let converted_fee: u128 = self
                .fee_per_weight
                .checked_mul(&fee)
                .or(None)?
                .try_into()
                .ok()?;
            // subtract weight from bought weight. Will not underflow because of `min()` above.
            self.weight -= weight;
            Some((Concrete(location.clone()), converted_fee).into())
        } else {
            None
        }
    }
}

fn can_put_capabilities<T: Config>(
    capabilities: &Vec<Capability<T::AccountId, BalanceOf<T>>>,
) -> DispatchResult {
    let mut occurance = [0u8; CAPABILITY_COUNT];
    for cap in capabilities {
        occurance[cap.as_usize()] += 1;
    }

    // ensure each capability only occurs once
    frame_support::ensure!(
        occurance.iter().all(|&x| x <= 1),
        Error::<T>::CapabilitiesNotPermitted
    );

    Ok(())
}

fn strict_capability_lookup<T: Config>(
    cap: Capability<T::AccountId, BalanceOf<T>>,
    capabilities: &[Capability<T::AccountId, BalanceOf<T>>],
) -> Result<Capability<T::AccountId, BalanceOf<T>>, Error<T>> {
    if capabilities.contains(&cap) {
        Ok(cap)
    } else {
        Err(Error::<T>::CapabilitiesNotPermitted)
    }
}

// We match with none and then return the capability with the value set in storage
fn soft_capability_lookup<T: Config>(
    cap: &Capability<T::AccountId, BalanceOf<T>>,
    capabilities: &[Capability<T::AccountId, BalanceOf<T>>],
) -> Result<Capability<T::AccountId, BalanceOf<T>>, Error<T>> {
    match cap {
        Capability::Teleport(_) => {
            match capabilities
                .iter()
                .find(|s| matches!(s, Capability::Teleport(_)))
            {
                None => Err(Error::<T>::CapabilitiesNotPermitted),
                Some(capability) => Ok(capability.clone()),
            }
        }
        Capability::Reserve(_) => {
            match capabilities
                .iter()
                .find(|s| matches!(s, Capability::Reserve(_)))
            {
                None => Err(Error::<T>::CapabilitiesNotPermitted),
                Some(capability) => Ok(capability.clone()),
            }
        }
        Capability::Payable { .. } => {
            match capabilities
                .iter()
                .find(|s| matches!(s, Capability::Payable { .. }))
            {
                None => Err(Error::<T>::CapabilitiesNotPermitted),
                Some(capability) => Ok(capability.clone()),
            }
        }
    }
}

/// map xcm instruction to the corresponding capability, ensuring each capability is only checked once
impl<AccountId, Balance, Call> TryFrom<&Instruction<Call>> for Capability<AccountId, Balance> {
    type Error = ();

    fn try_from(instruction: &Instruction<Call>) -> Result<Self, Self::Error> {
        match instruction {
            Instruction::WithdrawAsset(_)
            | Instruction::DepositAsset { .. }
            | Instruction::BuyExecution { .. } => Ok(Capability::Payable {
                fees_per_weight: None,
            }),
            _ => Err(()),
        }
    }
}
