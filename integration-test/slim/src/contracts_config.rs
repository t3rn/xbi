use crate::{Aura, Balances, Runtime, RuntimeEvent, Timestamp};
use frame_support::{parameter_types, traits::FindAuthor, weights::Weight};
use pallet_3vm_evm::{
    EnsureAddressNever, GasWeightMapping, StoredHashAddressMapping, SubstrateBlockHashMapping,
    ThreeVMCurrencyAdapter,
};
use pallet_3vm_evm_primitives::FeeCalculator;
use sp_core::{H160, U256};
use sp_runtime::{ConsensusEngineId, RuntimeAppPublic};

#[cfg(feature = "std")]
pub use pallet_3vm_evm_primitives::GenesisAccount as EvmGenesisAccount;

parameter_types! {
    pub const MaxValueSize: u32 = 16_384;
    pub const SS58Prefix: u16 = 42;
    pub const MaxCodeSize: u32 = 2 * 1024;
    pub SignalBounceThreshold: u32 = 5;
}

pub struct FindAuthorTruncated<F>(sp_std::marker::PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
    fn find_author<'a, I>(digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        if let Some(author_index) = F::find_author(digests) {
            let authority_id = Aura::authorities()[author_index as usize].clone();
            return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
        }
        None
    }
}

pub struct FreeGasWeightMapping;
impl GasWeightMapping for FreeGasWeightMapping {
    fn gas_to_weight(_gas: u64, without_base_weight: bool) -> Weight {
        Default::default()
    }

    fn weight_to_gas(_weight: Weight) -> u64 {
        0
    }
}

parameter_types! {
    pub const ChainId: u64 = 42;
    pub BlockGasLimit: U256 = U256::from(u32::max_value());
    pub PrecompilesValue: evm_precompile_util::Precompiles = evm_precompile_util::Precompiles::new(sp_std::vec![
        (0_u64, evm_precompile_util::KnownPrecompile::ECRecover),
        (1_u64, evm_precompile_util::KnownPrecompile::Sha256),
        (2_u64, evm_precompile_util::KnownPrecompile::Ripemd160),
        (3_u64, evm_precompile_util::KnownPrecompile::Identity),
        (4_u64, evm_precompile_util::KnownPrecompile::Modexp),
        (5_u64, evm_precompile_util::KnownPrecompile::Sha3FIPS256),
        (6_u64, evm_precompile_util::KnownPrecompile::Sha3FIPS512),
        (7_u64, evm_precompile_util::KnownPrecompile::ECRecoverPublicKey),
    ].into_iter().collect());
    pub WeightPerGas: Weight = Weight::from_ref_time(20_000);
}

impl pallet_3vm_evm::Config for Runtime {
    type AddressMapping = StoredHashAddressMapping<Self>;
    type BlockGasLimit = BlockGasLimit;
    type BlockHashMapping = SubstrateBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressNever<Self::AccountId>;
    type ChainId = ChainId;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type FeeCalculator = ();
    type FindAuthor = FindAuthorTruncated<Aura>;
    type GasWeightMapping = FreeGasWeightMapping;
    type OnChargeTransaction = ThreeVMCurrencyAdapter<Balances, ()>;
    type PrecompilesType = evm_precompile_util::Precompiles;
    type PrecompilesValue = PrecompilesValue;
    type Runner = pallet_3vm_evm::runner::stack::Runner<Self>;
    type ThreeVm = t3rn_primitives::threevm::NoopThreeVm;
    type WithdrawOrigin = EnsureAddressNever<Self::AccountId>;
    type Timestamp = Timestamp;
    type WeightPerGas = WeightPerGas;
    type OnCreate = ();
    type WeightInfo = ();
}
