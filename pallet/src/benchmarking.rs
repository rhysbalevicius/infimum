use super::*;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

use scale_info::prelude::string::String;
use sp_std::prelude::ToOwned;

use crate::Pallet as Infimum;

benchmarks!
{
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test)
}

