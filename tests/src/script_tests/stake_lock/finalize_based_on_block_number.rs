use super::utils::{build_context, ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER};
use crate::script_tests::utils::init_env_log;
use crate::script_tests::utils::layer1::{build_simple_tx_with_out_point, random_out_point};
use crate::script_tests::utils::rollup::random_always_success_script;
use ckb_error::assert_error_eq;
use ckb_script::ScriptError;
use ckb_types::{
    core::{Cycle, ScriptHashType},
    packed::{CellDep, CellInput, CellOutput, Script},
    prelude::*,
};
use gw_types::packed::StakeLockArgs;

// Transaction structure:
//
// ```
// CellDeps:
//   rollup_config_cell:
//   rollup_code_cell:
//   stake_code_cell:
//   rollup_state_cell:
//
// Inputs:
//   stake_state_cell:
//   stake_owner_cell:
//     Lock: <stake_state_cell.lock.args.StakeLockArgs.owner_lock_hash>
//
// Outputs:
//     <user defined>
// ```

// stake_block_number <= rollup_state_cell.last_finalized_block_number
#[test]
fn test_success_to_unlock_stake_via_finalize_based_on_block_number() {
    init_env_log();
    let (_, result) =
        unlock_stake_via_finalize_based_on_block_number(ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER);
    result.expect("success");
}

// stake_block_number > rollup_state_cell.last_finalized_block_number
#[test]
fn test_fail_to_unlock_stake_via_finalize_based_on_block_number() {
    init_env_log();
    let (stake_state_cell, result) = unlock_stake_via_finalize_based_on_block_number(
        ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER + 1,
    );
    let stake_code_type_hash: ckb_types::H256 = stake_state_cell.lock().code_hash().unpack();
    let expected_err = ScriptError::ValidationFailure(
        format!("by-type-hash/{}", stake_code_type_hash),
        45, // Error::NotFinalized
    )
    .input_lock_script(0);
    assert_error_eq!(expected_err, result.unwrap_err());
}

fn unlock_stake_via_finalize_based_on_block_number(
    stake_block_number: u64,
) -> (CellOutput, Result<Cycle, ckb_error::Error>) {
    let (
        mut ctx,
        rollup_state_out_point,
        rollup_state_type_script,
        stake_code_out_point,
        stake_code_type_script,
    ) = build_context();
    let rollup_state_type_hash = rollup_state_type_script.calc_script_hash();
    let stake_code_type_hash = stake_code_type_script.calc_script_hash();

    // Build a finalized stake_state_cell
    let stake_state_out_point = random_out_point();
    let stake_owner_lock_script = random_always_success_script();
    let stake_owner_lock_hash = stake_owner_lock_script.calc_script_hash();
    let stake_state_cell = CellOutput::new_builder()
        .lock(
            Script::new_builder()
                .code_hash(stake_code_type_hash.clone())
                .hash_type(ScriptHashType::Type.into())
                .args({
                    let stake_lock_args = StakeLockArgs::new_builder()
                        .stake_block_number(gw_types::prelude::Pack::pack(&stake_block_number))
                        .owner_lock_hash(gw_types::packed::Byte32::new_unchecked(
                            stake_owner_lock_hash.as_bytes(),
                        ))
                        .build();
                    let mut args = Vec::new();
                    args.extend_from_slice(&rollup_state_type_hash.as_slice());
                    args.extend_from_slice(&stake_lock_args.as_slice());
                    args.pack()
                })
                .build(),
        )
        .build();

    // Build stake_owner_cell
    let stake_owner_cell = CellOutput::new_builder()
        .lock(stake_owner_lock_script)
        .build();
    let stake_owner_out_point = ctx.insert_cell(stake_owner_cell, Default::default());

    // Build transaction
    let tx = build_simple_tx_with_out_point(
        &mut ctx.inner,
        (stake_state_cell.clone(), Default::default()),
        stake_state_out_point,
        (CellOutput::new_builder().build(), Default::default()),
    )
    .as_advanced_builder()
    .input(CellInput::new(stake_owner_out_point, 0))
    .cell_dep(
        CellDep::new_builder()
            .out_point(rollup_state_out_point)
            .build(),
    )
    .cell_dep(
        CellDep::new_builder()
            .out_point(stake_code_out_point)
            .build(),
    )
    .cell_dep(ctx.rollup_config_dep.clone())
    .cell_dep(ctx.always_success_dep.clone())
    .build();
    let result = ctx.verify_tx(tx);
    (stake_state_cell, result)
}
