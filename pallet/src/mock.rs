use crate::*;
use crate as pallet_infimum;
use frame_support::{
    derive_impl,
	traits::{ConstU8, ConstU32, ConstU64}
};
use sp_core::H256;
use frame_support::pallet_prelude::PhantomData;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Infimum: pallet_infimum::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type Hash = H256;
	type RuntimeCall = RuntimeCall;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl Config for Test {
    type MaxCoordinatorPolls = ConstU32<1028>;
    type MaxVerifyKeyLength = ConstU32<4096>;
    type MaxTreeArity = ConstU8<16>;
    type MinTreeArity = ConstU8<2>;
    type MaxTreeDepth = ConstU8<255>;
    type MaxVoteOptions = ConstU32<32>;
    type MaxPollRegistrations = ConstU32<65536>;
    type MaxPollInteractions = ConstU32<65536>;
    type MaxIterationDepth = ConstU32<256>;
	type RuntimeEvent = RuntimeEvent;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = RuntimeGenesisConfig {
		system: Default::default(),
		infimum: pallet_infimum::GenesisConfig {
            _marker: PhantomData
		},
	}
	.build_storage()
	.unwrap();
	t.into()
}
