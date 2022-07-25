#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

pub use pallet::*;

pub use xcm::latest;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    pub use crate::weights::WeightInfo;

    use pallet_xbi_portal::{primitives::xbi::XBIPortal, xbi_format::XBIFormat};

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use sp_std::default::Default;
    use xcm::latest::prelude::*;

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
            xbi_batch: Vec<XBIFormat>,
            dest_para_id: cumulus_primitives_core::ParaId,
        ) -> DispatchResultWithPostInfo {
            Self::do_batch_enter_xbi(xbi_batch, dest_para_id)
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn do_batch_enter_xbi(
            xbi_batch: Vec<XBIFormat>,
            dest_para_id: cumulus_primitives_core::ParaId,
        ) -> DispatchResultWithPostInfo {
            for xbi in xbi_batch {
                let _versioned_multi_loc = Box::new(
                    xcm::VersionedMultiLocation::try_from(Parachain(dest_para_id.into()).into())
                        .map_err(|_e| Error::<T>::CannotTransformParaId)?,
                );

                T::XBIPortal::do_check_in_xbi(xbi).map_err(|_e| Error::<T>::CannotEnterXBI)?;
            }

            Ok(().into())
        }

        pub fn on_xbi_callback(
            xbi_checkin: pallet_xbi_portal::xbi_codec::XBICheckIn<T::BlockNumber>,
            xbi_checkout: pallet_xbi_portal::xbi_codec::XBICheckOut,
        ) -> DispatchResultWithPostInfo {
            let id = xbi_checkin.xbi.metadata.id.clone();



            Ok(().into())
        }


        pub fn resolve_sfx_2_xbi(
            xbi_checkin: pallet_xbi_portal::xbi_codec::XBICheckIn<T::BlockNumber>,
            xbi_checkout: pallet_xbi_portal::xbi_codec::XBICheckOut,
        ) -> DispatchResultWithPostInfo {
            let id = xbi_checkin.xbi.metadata.id.clone();


            Ok(().into())
        }

        pub fn resolve_xbi_2_sfx(
            xbi_checkin: pallet_xbi_portal::xbi_codec::XBICheckIn<T::BlockNumber>,
            xbi_checkout: pallet_xbi_portal::xbi_codec::XBICheckOut,
        ) -> DispatchResultWithPostInfo {
            let id = xbi_checkin.xbi.metadata.id.clone();

            Ok(().into())
        }

        pub fn t3rn_sfx_resolver(
            xbi_checkin: pallet_xbi_portal::xbi_codec::XBICheckIn<T::BlockNumber>,
            xbi_checkout: pallet_xbi_portal::xbi_codec::XBICheckOut,
        ) -> DispatchResultWithPostInfo {

            let id = xbi_checkin.xbi.metadata.id.clone();


            Ok(().into())
        }
    }
}
