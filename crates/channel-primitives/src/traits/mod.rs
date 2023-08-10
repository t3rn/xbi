use codec::{Decode, Encode, FullCodec};
use frame_support::traits::{fungibles::Inspect, tokens::Preservation, Get};
use sp_runtime::DispatchResult;
use sp_std::prelude::*;
use xp_format::Fees;

/// A set of traits containing some loosely typed shims to storage interactions in substrate.
///
/// NOTE: the shims in this module are largely so that we can have a slim interface for some genric queues
/// without relying on `frame`. Ideally, these structures would be extracted out of frame in the future.
/// But for now we limit to what we have.
///
/// Due to the above limitations, some functionality is omitted, since it isn't required for our use case.
pub mod shims;

/// A structure containing the output of an instruction handle. This should be used to hold any results and error information.
/// Which might be relevant to the user.
///
/// This also adds information about weight used by the instruction handler.
#[derive(Encode, Decode, Default, Debug, Clone, PartialEq, Eq)]
pub struct HandlerInfo<Weight: core::fmt::Debug> {
    // TODO[Optimisation]: We can bound the size, but ideally this should be configured by the user who sends the message.
    // We have ideas on how to specify this in future releases.
    pub output: Vec<u8>,
    // The weight that was used to handle the message.
    pub weight: Weight,
}

impl<Weight: core::fmt::Debug> From<(Vec<u8>, Weight)> for HandlerInfo<Weight> {
    fn from(t: (Vec<u8>, Weight)) -> Self {
        let (bytes, i) = t;
        HandlerInfo {
            output: bytes,
            weight: i,
        }
    }
}

// Justification: no need for from here
#[allow(clippy::from_over_into)]
#[cfg(feature = "frame")]
impl Into<frame_support::dispatch::PostDispatchInfo>
    for HandlerInfo<frame_support::weights::Weight>
{
    fn into(self) -> frame_support::dispatch::PostDispatchInfo {
        frame_support::dispatch::PostDispatchInfo {
            actual_weight: Some(self.weight),
            pays_fee: frame_support::dispatch::Pays::Yes,
        }
    }
}

/// A simple trait that allows a parachain to specify how they would handle an xbi instruction.
///
/// This is also utilised as a simple gateway for routing messages within a parachain, and could be used for different pallets to contact each other.
///
/// Note: This would currently need runtime upgrades to support new/less functionality, however there are plans to make this routing layer on-chain.
// TODO: a result validator shoulld also allow a sender of a message to validate what they deem as a successful result, otherwise the fallback is on the parachain to prove the message was handled correctly.
#[cfg(feature = "frame")]
pub trait XbiInstructionHandler<Origin> {
    fn handle(
        origin: &Origin,
        xbi: &mut xp_format::XbiFormat,
    ) -> Result<
        HandlerInfo<frame_support::weights::Weight>,
        frame_support::dispatch::DispatchErrorWithPostInfo,
    >;
}

/// A trait providing generic write access, its intention is so that a pallet may provide a way to write channel elements to storage.
pub trait Writable<T: FullCodec> {
    fn write(t: T) -> DispatchResult;
}

#[cfg(feature = "frame")]
pub trait ChargeForMessage<AccountId, Currency, Assets, Custodian>
where
    Currency: frame_support::traits::ReservableCurrency<AccountId>,
    Assets: frame_support::traits::fungibles::Mutate<AccountId>,
    Custodian: Get<AccountId>,
{
    // Just a 1:1 fixed weigth -> fee for the default impl
    fn charge(origin: &AccountId, fees: &Fees) -> DispatchResult {
        if let Some(asset) = fees.asset {
            let asset: <Assets as Inspect<AccountId>>::AssetId =
                Decode::decode(&mut &asset.encode()[..])
                    .map_err(|_| "Failed to decode asset from fees")?;

            let balance: <Assets as Inspect<AccountId>>::Balance =
                Decode::decode(&mut &fees.get_aggregated_limit().encode()[..])
                    .map_err(|_| "Failed to decode balance from fees")?;

            if let Err(x) = Assets::can_withdraw(asset.clone(), origin, balance).into_result(true) {
                log::warn!(target: "xp-channel", "Insufficient funds to pay fees, {:?}", x);
                return Err(x)
            }
            Assets::transfer(
                asset.clone(),
                origin,
                &Custodian::get(),
                balance,
                Preservation::Preserve,
            )?;

            log::debug!(target: "xp-channel", "Charged Asset({:?}, {:?}) for XBI metadata fees {:?}", asset, balance, fees);
        } else {
            let balance: Currency::Balance =
                Decode::decode(&mut &fees.get_aggregated_limit().encode()[..])
                    .map_err(|_| "Failed to decode balance from fee")?;

            // TODO: ensure that the balance is sufficient
            Currency::reserve(origin, balance)?;
            log::debug!(target: "xp-channel", "Charged {:?} for XBI metadata fees {:?}", balance, fees);
        }
        Ok(())
    }
}

#[cfg(feature = "frame")]
pub trait RefundForMessage<AccountId, Currency, Assets, Custodian>
where
    Currency: frame_support::traits::ReservableCurrency<AccountId>,
    Assets: frame_support::traits::fungibles::Mutate<AccountId>,
    Custodian: Get<AccountId>,
{
    fn refund(origin: &AccountId, fees: &Fees) -> DispatchResult {
        if let Some(asset) = fees.asset {
            let asset: <Assets as Inspect<AccountId>>::AssetId =
                Decode::decode(&mut &asset.encode()[..])
                    .map_err(|_| "Failed to decode asset from fees")?;

            let cost: <Assets as Inspect<AccountId>>::Balance =
                Decode::decode(&mut &fees.get_aggregated_cost().encode()[..])
                    .map_err(|_| "Failed to decode balance from fees")?;

            let reserved: <Assets as Inspect<AccountId>>::Balance =
                Decode::decode(&mut &fees.get_aggregated_limit().encode()[..])
                    .map_err(|_| "Failed to decode balance from fees")?;

            if cost < reserved {
                let to_unreserve: <Assets as Inspect<AccountId>>::Balance =
                    Decode::decode(&mut &(reserved - cost).encode()[..])
                        .map_err(|_| "Failed to decode balance from aggregation")?;

                Assets::transfer(
                    asset,
                    &Custodian::get(),
                    origin,
                    to_unreserve,
                    Preservation::Preserve,
                )?;
            } else {
                log::warn!(target: "xp-channel", "Tried refunding more than was reserved for XBI metadata fees {:?} {:?}", cost, reserved);
            }
        } else {
            let cost = fees.get_aggregated_cost();
            let reserved = fees.get_aggregated_limit();
            if cost < reserved {
                let to_unreserve: Currency::Balance =
                    Decode::decode(&mut &(reserved - cost).encode()[..])
                        .map_err(|_| "Failed to decode balance from aggregation")?;
                Currency::unreserve(origin, to_unreserve);
            } else {
                log::warn!(target: "xp-channel", "Tried refunding more than was reserved for XBI metadata fees {:?} {:?}", cost, reserved);
            }
        }
        Ok(())
    }
}

#[cfg(feature = "frame")]
pub trait MonetaryForMessage<AccountId, Currency, Assets, Custodian>:
    ChargeForMessage<AccountId, Currency, Assets, Custodian>
    + RefundForMessage<AccountId, Currency, Assets, Custodian>
where
    Currency: frame_support::traits::ReservableCurrency<AccountId>,
    Assets: frame_support::traits::fungibles::Mutate<AccountId>,
    Custodian: Get<AccountId>,
{
}

impl<AccountId, Currency, Assets, Custodian>
    ChargeForMessage<AccountId, Currency, Assets, Custodian> for ()
where
    Currency: frame_support::traits::ReservableCurrency<AccountId>,
    Assets: frame_support::traits::fungibles::Mutate<AccountId>,
    Custodian: Get<AccountId>,
{
}

impl<AccountId, Currency, Assets, Custodian>
    RefundForMessage<AccountId, Currency, Assets, Custodian> for ()
where
    Currency: frame_support::traits::ReservableCurrency<AccountId>,
    Assets: frame_support::traits::fungibles::Mutate<AccountId>,
    Custodian: Get<AccountId>,
{
}

impl<AccountId, Currency, Assets, Custodian>
    MonetaryForMessage<AccountId, Currency, Assets, Custodian> for ()
where
    Currency: frame_support::traits::ReservableCurrency<AccountId>,
    Assets: frame_support::traits::fungibles::Mutate<AccountId>,
    Custodian: Get<AccountId>,
{
}
