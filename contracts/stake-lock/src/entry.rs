//! Stake-lock

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
// use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use crate::ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::Unpack as CKBTypeUnpack},
    high_level::load_script,
};

use gw_utils::cells::{
    rollup::{load_rollup_config, search_rollup_cell, search_rollup_state},
    utils::search_lock_hash,
};
use gw_utils::finality::{is_finalized_based_on_block_number, is_finalized_based_on_timestamp};
use gw_utils::gw_types;

use gw_types::{
    core::Timepoint,
    packed::{StakeLockArgs, StakeLockArgsReader},
    prelude::*,
};

use crate::error::Error;

/// args: rollup_type_hash | stake lock args
fn parse_lock_args() -> Result<([u8; 32], StakeLockArgs), Error> {
    let mut rollup_type_hash = [0u8; 32];
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    if args.len() < rollup_type_hash.len() {
        return Err(Error::InvalidArgs);
    }
    rollup_type_hash.copy_from_slice(&args[..32]);
    match StakeLockArgsReader::verify(&args.slice(32..), false) {
        Ok(()) => Ok((
            rollup_type_hash,
            StakeLockArgs::new_unchecked(args.slice(32..)),
        )),
        Err(_) => Err(Error::InvalidArgs),
    }
}

pub fn main() -> Result<(), Error> {
    let (rollup_type_hash, lock_args) = parse_lock_args()?;

    // Unlock by User
    // read global state from rollup cell in deps
    if let Some(global_state) = search_rollup_state(&rollup_type_hash, Source::CellDep)? {
        // check if the stake cells are finalized
        let config = load_rollup_config(&global_state.rollup_config_hash().unpack())?;
        let timepoint = Timepoint::from_full_value(lock_args.stake_block_number().unpack());
        if timepoint.is_timestamp_based() {
            let timestamp = timepoint.value();
            if !is_finalized_based_on_timestamp(&config, &rollup_type_hash.pack(), timestamp, Source::CellDep)? {
                return Err(Error::NotFinalized);
            }
        } else {
            let block_number = timepoint.value();
            if !is_finalized_based_on_block_number(&global_state, block_number) {
                return Err(Error::NotFinalized);
            }
        }

        // 2. check if owner_lock_hash exists in input cells
        if search_lock_hash(&lock_args.owner_lock_hash().unpack(), Source::Input).is_some() {
            return Ok(());
        }
    }

    // Unlock by Rollup cell
    // check if rollup cell exists in the inputs, the following verification will be handled
    // by rollup state validator.
    if search_rollup_cell(&rollup_type_hash, Source::Input).is_some() {
        return Ok(());
    }

    Err(Error::InvalidStakeCellUnlock)
}
