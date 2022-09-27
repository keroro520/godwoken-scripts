use crate::{cells::rollup::search_rollup_cell, error::Error};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_header, QueryIter},
};
use gw_types::core::Timepoint;
use gw_types::packed::{Byte32, GlobalState, RollupConfig};
use gw_types::prelude::{Entity, Unpack};

/// Determines if the given `timestamp` is finalized based on the finality rule.
///
/// See also: https://talk.nervos.org/t/optimize-godwoken-finality-and-on-chain-cost/6739
pub fn is_finalized_based_on_timestamp(
    rollup_config: &RollupConfig,
    rollup_type_hash: &Byte32,
    timestamp: u64,
    rollup_cell_source: Source,
) -> Result<bool, Error> {
    let rollup_cell_timestamp = obtain_timestamp_of_rollup_cell(rollup_type_hash, rollup_cell_source)?;
    let max_header_timestamp = obtain_max_timestamp_of_header_deps().unwrap_or_default();
    let l1_timestamp = if rollup_cell_timestamp > max_header_timestamp {
        rollup_cell_timestamp
    } else {
        max_header_timestamp
    };
    let finality_timepoint = Timepoint::from_full_value(rollup_config.finality_blocks().unpack());
    let finality_duration_ms = finality_duration_ms(finality_timepoint);
    debug!(
        "[is_finalized_based_on_timestamp] is_finalized: {}, l1_timestamp: {}, timestamp: {}, finality_duration_ms: {}",
        l1_timestamp >= timestamp + finality_duration_ms, l1_timestamp, timestamp, finality_duration_ms
    );
    Ok(l1_timestamp >= timestamp + finality_duration_ms)
}

pub fn is_finalized_based_on_block_number(global_state: &GlobalState, block_number: u64) -> bool {
    let last_finalized_block_number: u64 = global_state.last_finalized_block_number().unpack();
    block_number <= last_finalized_block_number
}

pub fn finality_duration_ms(finality_timepoint: Timepoint) -> u64 {
    if finality_timepoint.is_block_number_based() {
        // 7 * 24 * 60 * 60 / 16800 * 1000 = 36000
        const BLOCK_INTERVAL_IN_MILLISECONDS: u64 = 36000;
        finality_timepoint.value() * BLOCK_INTERVAL_IN_MILLISECONDS
    } else {
        finality_timepoint.value()
    }
}

/// Obtain the timestamp of the input rollup cell included block.
fn obtain_timestamp_of_rollup_cell(rollup_type_hash: &Byte32, source: Source) -> Result<u64, Error> {
    let mut rollup_type_hash_array = [0u8; 32];
    rollup_type_hash_array.copy_from_slice(&rollup_type_hash.as_slice());

    let index = search_rollup_cell(&rollup_type_hash_array, source)
        .ok_or(Error::RollupCellNotFound)?;
    let header = load_header(index, source)?;

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
