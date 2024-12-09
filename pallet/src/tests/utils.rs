use crate::{
    mock::*
};
use frame_support::pallet_prelude::Hooks;

pub fn run_to_block(n: u64)
{
    while System::block_number() < n
    {
        if System::block_number() > 1 
        {
            Infimum::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Infimum::on_initialize(System::block_number());
    }
}
