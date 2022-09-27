use crate::script_tests::utils::rollup::{random_always_success_script, CellContext};
use crate::testing_tool::programs::{ALWAYS_SUCCESS_CODE_HASH, STAKE_LOCK_PROGRAM};
use ckb_types::{
    packed::{CellOutput, OutPoint, Script},
    prelude::*,
};
use gw_types::bytes::Bytes;
use gw_types::packed::{GlobalState, RollupConfig};

const BLOCK_INTERVAL_IN_MILLISECONDS: u64 = 36000;
pub(super) const ROLLUP_CONFIG_FINALITY_BLOCKS: u64 = 1;
pub(super) const ROLLUP_CONFIG_FINALITY_DURATION_MS: u64 =
    ROLLUP_CONFIG_FINALITY_BLOCKS * BLOCK_INTERVAL_IN_MILLISECONDS;
pub(super) const ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER: u64 = 100;
pub(super) const ROLLUP_STATE_CELL_TIMESTAMP: u64 = 1555204979310;

// Build common-used cells for testing stake-lock:
//   - rollup_config_cell, finality_blocks = ROLLUP_CONFIG_FINALITY_BLOCKS
//   - rollup_code_cell, is ALWAYS_SUCCESS_PROGRAM
//   - rollup_state_cell, last_finalized_block_number = ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER
//   - stake_code_cell, is STAKE_LOCK_PROGRAM
//
// Return (ctx, rollup_state_out_point, rollup_state_type_script, stake_code_out_point, stake_code_type_script);
pub(super) fn build_context() -> (CellContext, OutPoint, Script, OutPoint, Script) {
    let rollup_config = RollupConfig::new_builder()
        .finality_blocks(gw_types::prelude::Pack::pack(
            &ROLLUP_CONFIG_FINALITY_BLOCKS,
        ))
        .build();
    let mut ctx = CellContext::new(&rollup_config, Default::default());

    // Build a always-success rollup_state_cell, because we are testing
    // stake-lock only
    let rollup_state_data = GlobalState::new_builder()
        .last_finalized_block_number(gw_types::prelude::Pack::pack(
            &ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER,
        ))
        .rollup_config_hash({
            let rollup_config_data_hash = CellOutput::calc_data_hash(&rollup_config.as_bytes());
            gw_types::packed::Byte32::new_unchecked(rollup_config_data_hash.as_bytes())
        })
        .build();
    let rollup_state_cell = CellOutput::new_builder()
        .lock(random_always_success_script())
        .type_(Some(random_always_success_script()).pack())
        .build();
    let rollup_state_type_script = rollup_state_cell
        .type_()
        .to_opt()
        .expect("should be always-success");
    let rollup_state_out_point = ctx.insert_cell(rollup_state_cell, rollup_state_data.as_bytes());

    // Build costodian_code_cell
    let stake_code_data = STAKE_LOCK_PROGRAM.clone();
    let stake_code_cell = CellOutput::new_builder()
        .lock(random_always_success_script())
        .type_(Some(random_always_success_script()).pack())
        .build();
    let stake_code_type_script = stake_code_cell
        .type_()
        .to_opt()
        .expect("should be always-success");
    let stake_code_out_point = ctx.insert_cell(stake_code_cell, stake_code_data);

    return (
        ctx,
        rollup_state_out_point,
        rollup_state_type_script,
        stake_code_out_point,
        stake_code_type_script,
    );
}
