#![feature(associated_type_defaults)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

pub use pallet::*;

pub use xcm::latest;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

pub mod t3rn_sfx;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    pub use crate::weights::WeightInfo;

    use pallet_xbi_portal::{primitives::xbi::XBIPortal, xbi_format::XbiFormat};

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use sp_std::{default::Default, prelude::*};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type XBIPortal: XBIPortal<Self>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        CannotTransformParaId,
        CannotEnterXBI,
        XBIPluginUnavailable,
        EnterSfxDecodingValueErr,
        EnterSfxDecodingAddressErr,
        EnterSfxDecodingDataErr,
        EnterSfxNotRecognized,
        ExitOnlyXBIResultResolvesToSFXConfirmation,
    }

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000)]
        pub fn batch_enter_xbi(
            _origin: OriginFor<T>, // Active relayer
            xbi_batch: Vec<XbiFormat>,
        ) -> DispatchResultWithPostInfo {
            Self::do_batch_enter_xbi(xbi_batch)
        }

        #[pallet::weight(100_000)]
        pub fn enter_xbi(
            _origin: OriginFor<T>, // Active relayer
            xbi: XbiFormat,
        ) -> DispatchResultWithPostInfo {
            Self::do_enter_xbi(xbi)
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn do_batch_enter_xbi(xbi_batch: Vec<XbiFormat>) -> DispatchResultWithPostInfo {
            for xbi in xbi_batch {
                T::XBIPortal::do_check_in_xbi(xbi).map_err(|_e| Error::<T>::CannotEnterXBI)?;
            }
            Ok(().into())
        }

        pub fn do_enter_xbi(xbi: XbiFormat) -> DispatchResultWithPostInfo {
            T::XBIPortal::do_check_in_xbi(xbi).map_err(|_e| Error::<T>::CannotEnterXBI)?;
            Ok(().into())
        }
    }
}
