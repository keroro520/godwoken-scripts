// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use crate::{
    ckb_std::{
        ckb_constants::Source,
        ckb_types::{bytes::Bytes, prelude::Unpack as CKBUnpack},
        debug,
        high_level::load_script,
        syscalls::load_cell_data,
    },
    eth_signature::{extract_eth_lock_args, EthAddress, Secp256k1Eth},
};
use gw_utils::{
    cells::utils::search_lock_hash, ckb_std::high_level::load_witness_args, error::Error,
    gw_common::H256,
};

/// Eth account lock
/// script args: rollup_script_hash(32 bytes) | eth_address(20 bytes)
/// data: owner_lock_hash(32 bytes) | message(32 bytes)
pub fn main() -> Result<(), Error> {
    // parse args
    let script = load_script()?;
    let args: Bytes = CKBUnpack::unpack(&script.args());
    let (_rollup_script_hash, eth_address) = extract_eth_lock_args(args)?;
    debug!("eth_address {:?}", &eth_address);

    // parse data
    let (owner_lock_hash, message) = parse_data()?;

    // check owner lock hash cell
    // to prevent others unlock this cell
    if search_lock_hash(&owner_lock_hash, Source::Input).is_none() {
        return Err(Error::OwnerCellNotFound);
    }

    // verify signature
    debug!("Verify message signature {:?}", &message);
    verify_message_signature(eth_address, message)?;

    Ok(())
}

/// load signature from witness
fn load_signature_from_witness() -> Result<[u8; 65], Error> {
    const SIGNATURE_SIZE: usize = 65;

    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let signature: Bytes = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::WrongSignature)?
        .unpack();
    if signature.len() != SIGNATURE_SIZE {
        debug!(
            "signature len: {}, expected len: {}",
            signature.len(),
            SIGNATURE_SIZE
        );
        return Err(Error::WrongSignature);
    }

    let mut buf = [0u8; 65];
    buf.copy_from_slice(&signature);
    Ok(buf)
}

fn verify_message_signature(eth_address: EthAddress, message: H256) -> Result<(), Error> {
    // load signature
    let signature = load_signature_from_witness()?;
    // verify message
    let secp256k1_eth = Secp256k1Eth::default();
    let valid = secp256k1_eth.verify_message(eth_address, signature, message)?;
    if !valid {
        debug!("Wrong signature, message: {:?}", message);
        return Err(Error::WrongSignature);
    }
    Ok(())
}

/// parse cell's data
/// return (owner_lock_hash, message)
fn parse_data() -> Result<([u8; 32], H256), Error> {
    let mut data = [0u8; 64];
    let loaded_size = load_cell_data(&mut data, 0, 0, Source::GroupInput)?;

    if loaded_size == 64 {
        // copy owner lock hash
        let mut owner_lock_hash = [0u8; 32];
        owner_lock_hash.copy_from_slice(&data[..32]);
        // copy message
        let mut msg = [0u8; 32];
        msg.copy_from_slice(&data[32..]);
        Ok((owner_lock_hash, msg.into()))
    } else {
        debug!("Invalid data size: {}", loaded_size);
        Err(Error::Encoding)
    }
}
