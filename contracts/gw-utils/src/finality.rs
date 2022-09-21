use crate::{cells::rollup::search_rollup_cell, error::Error};
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_lock, load_header, QueryIter},
};
use gw_types::core::{ScriptHashType, Timepoint};
use gw_types::packed::{Byte32, RollupConfig};
use gw_types::prelude::{Entity, Unpack};

/// Determines if the given `timestamp` is finalized based on the finality rule.
///
/// See also: https://talk.nervos.org/t/optimize-godwoken-finality-and-on-chain-cost/6739
pub fn is_finalized(
    rollup_config: &RollupConfig,
    rollup_type_hash: &Byte32,
    timestamp: u64,
) -> Result<bool, Error> {
    let rollup_cell_timestamp = obtain_timestamp_of_rollup_cell(rollup_type_hash)?;
    let max_header_timestamp = obtain_max_timestamp_of_header_deps().unwrap_or_default();
    let l1_timestamp = if rollup_cell_timestamp > max_header_timestamp {
        rollup_cell_timestamp
    } else {
        max_header_timestamp
    };
    let finality_blocks: u64 = rollup_config.finality_blocks().unpack();
    let finality_timepoint = Timepoint::from_full_value(finality_blocks);
    let finality_duration_in_secs = finality_duration_in_secs(finality_timepoint);
    Ok(l1_timestamp >= timestamp + finality_duration_in_secs)
}

fn finality_duration_in_secs(finality_timepoint: Timepoint) -> u64 {
    if finality_timepoint.is_block_number_based() {
        // 7 * 24 * 60 * 60 / 16800 = 36
        const BLOCK_INTERVAL_IN_SECONDS: u64 = 36;
        finality_timepoint.value() * BLOCK_INTERVAL_IN_SECONDS
    } else {
        finality_timepoint.value()
    }
}

/// Obtain the timestamp of the input rollup cell included block.
fn obtain_timestamp_of_rollup_cell(rollup_type_hash: &Byte32) -> Result<u64, Error> {
    let mut rollup_type_hash_array = [0u8; 32];
    rollup_type_hash_array.copy_from_slice(&rollup_type_hash.as_slice());

    let index = search_rollup_cell(&rollup_type_hash_array, Source::Input)
        .ok_or(Error::RollupCellNotFound)?;
    let header = load_header(index, Source::Input)?;

    let mut buf = [0u8; 8];
    buf.copy_from_slice(header.raw().timestamp().as_slice());
    let timestamp: u64 = u64::from_le_bytes(buf);
    return Ok(timestamp);
}

/// Obtain the max timestamp of the header-deps
fn obtain_max_timestamp_of_header_deps() -> Option<u64> {
    let mut buf = [0u8; 8];
    QueryIter::new(load_header, Source::HeaderDep)
        .map(|header| {
            buf.copy_from_slice(header.raw().timestamp().as_slice());
            let timestamp: u64 = u64::from_le_bytes(buf);
            timestamp
        })
        .max()
}

/// Obtain the maximum timestamp of input cells that use the given lock script
pub fn obtain_max_timestmap_via_lock_script(
    rollup_type_hash: &Byte32,
    lock_script_type_hash: &Byte32,
) -> Result<Option<u64>, Error> {
    let mut max_timestamp = None;
    let mut buf = [0u8; 8];
    for index in query_indexes_via_lock_script(rollup_type_hash, lock_script_type_hash) {
        let header = load_header(index, Source::Input)?;
        let timestamp = {
            buf.copy_from_slice(header.raw().timestamp().as_slice());
            let timestamp: u64 = u64::from_le_bytes(buf);
            timestamp
        };
        if Some(timestamp) > max_timestamp {
            max_timestamp = Some(timestamp);
        }
    }
    Ok(max_timestamp)
}

fn query_indexes_via_lock_script(
    rollup_type_hash: &Byte32,
    lock_script_type_hash: &Byte32,
) -> Vec<usize> {
    QueryIter::new(load_cell_lock, Source::Input)
        .enumerate()
        .filter_map(|(index, lock)| {
            let lock_args = lock.args();
            let is_matched = lock_args.len() > 32
                && &lock_args.as_slice()[..32] == rollup_type_hash.as_slice()
                && lock.code_hash().as_slice() == lock_script_type_hash.as_slice()
                && lock.hash_type() == ScriptHashType::Type.into();
            if is_matched {
                Some(index)
            } else {
                None
            }
        })
        .collect()
}
