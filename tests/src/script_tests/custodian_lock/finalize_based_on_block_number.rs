use super::utils::{build_context, ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER};
use crate::script_tests::utils::init_env_log;
use crate::script_tests::utils::layer1::{build_simple_tx_with_out_point, random_out_point};
use ckb_error::assert_error_eq;
use ckb_script::ScriptError;
use ckb_types::{
    core::{Cycle, ScriptHashType},
    packed::{CellDep, CellInput, CellOutput, Script},
    prelude::*,
};
use gw_types::packed::CustodianLockArgs;

// Transaction structure:
//
// ```
// CellDeps:
//   rollup_config_cell:
//   rollup_code_cell:
//   custodian_code_cell:
//
// Inputs:
//   rollup_state_cell:
//   custodian_state_cell:
//
// Outputs:
//   <user defined>:
// ```

// deposit_block_number <= rollup_state_cell.last_finalized_block_number
#[test]
fn test_success_to_unlock_custodian_via_finalize_based_on_block_number() {
    init_env_log();
    let (_, result) = unlock_custodian_via_finalize_based_on_block_number(
        ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER,
    );
    result.expect("success");
}

// deposit_block_number > rollup_state_cell.last_finalized_block_number
#[test]
fn test_fail_to_unlock_custodian_via_finalize_based_on_block_number() {
    init_env_log();
    let (custodian_state_cell, result) = unlock_custodian_via_finalize_based_on_block_number(
        ROLLUP_STATE_LAST_FINALIZED_BLOCK_NUMBER + 1,
    );
    let custodian_code_type_hash: ckb_types::H256 =
        custodian_state_cell.lock().code_hash().unpack();
    let expected_err = ScriptError::ValidationFailure(
        format!("by-type-hash/{}", custodian_code_type_hash),
        1, // Error::IndexOutOfBound
    )
    .input_lock_script(0);
    assert_error_eq!(expected_err, result.unwrap_err());
}

fn unlock_custodian_via_finalize_based_on_block_number(
    deposit_block_number: u64,
) -> (CellOutput, Result<Cycle, ckb_error::Error>) {
    let (
        mut ctx,
        rollup_state_out_point,
        rollup_state_type_script,
        custodian_code_out_point,
        custodian_code_type_script,
    ) = build_context();
    let rollup_state_type_hash = rollup_state_type_script.calc_script_hash();
    let custodian_code_type_hash = custodian_code_type_script.calc_script_hash();

    // Build a finalized custodian_state_cell
    let custodian_state_out_point = random_out_point();
    let custodian_state_cell = CellOutput::new_builder()
        .lock(
            Script::new_builder()
                .code_hash(custodian_code_type_hash.clone())
                .hash_type(ScriptHashType::Type.into())
                .args({
                    let custodian_lock_args = CustodianLockArgs::new_builder()
                        .deposit_block_number(gw_types::prelude::Pack::pack(&deposit_block_number))
                        .build();
                    let mut args = Vec::new();
                    args.extend_from_slice(&rollup_state_type_hash.as_slice());
                    args.extend_from_slice(&custodian_lock_args.as_slice());
                    args.pack()
                })
                .build(),
        )
        .build();

    // Build transaction
    let tx = build_simple_tx_with_out_point(
        &mut ctx.inner,
        (custodian_state_cell.clone(), Default::default()),
        custodian_state_out_point,
        (CellOutput::new_builder().build(), Default::default()),
    )
    .as_advanced_builder()
    .input(CellInput::new(rollup_state_out_point, 0))
    .cell_dep(
        CellDep::new_builder()
            .out_point(custodian_code_out_point)
            .build(),
    )
    .cell_dep(ctx.rollup_config_dep.clone())
    .cell_dep(ctx.always_success_dep.clone())
    .build();
    let result = ctx.verify_tx(tx);
    (custodian_state_cell, result)
}
