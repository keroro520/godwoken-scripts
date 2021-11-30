use blake2b::new_blake2b;
use ckb_types::bytes::Bytes;
use gw_common::blake2b;
use lazy_static::lazy_static;
use std::{fs, io::Read, path::PathBuf};

const SCRIPT_DIR: &str = "../build/debug";
const CHALLENGE_LOCK_PATH: &str = "challenge-lock";
const WITHDRAWAL_LOCK_PATH: &str = "withdrawal-lock";
const STATE_VALIDATOR: &str = "state-validator";
const ALWAYS_SUCCESS_PATH: &str = "always-success";
const SECP256K1_DATA_PATH: &str = "../c/deps/ckb-production-scripts/build/secp256k1_data";
const C_SCRIPTS_DIR: &str = "../../godwoken-scripts/c/build";
const META_CONTRACT_BIN_NAME: &str = "meta-contract-validator";
// account locks
const ETH_LOCK_PATH: &str = "eth-account-lock";
const TRON_LOCK_PATH: &str = "tron-account-lock";

lazy_static! {
    pub static ref ALWAYS_SUCCESS_PROGRAM: Bytes = {
        let mut buf = Vec::new();
        let mut path = PathBuf::new();
        path.push(&SCRIPT_DIR);
        path.push(&ALWAYS_SUCCESS_PATH);
        let mut f = fs::File::open(&path).expect("load program");
        f.read_to_end(&mut buf).expect("read program");
        Bytes::from(buf.to_vec())
    };
    pub static ref ALWAYS_SUCCESS_CODE_HASH: [u8; 32] = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&ALWAYS_SUCCESS_PROGRAM);
        hasher.finalize(&mut buf);
        buf
    };
    pub static ref CHALLENGE_LOCK_PROGRAM: Bytes = {
        let mut buf = Vec::new();
        let mut path = PathBuf::new();
        path.push(&SCRIPT_DIR);
        path.push(&CHALLENGE_LOCK_PATH);
        let mut f = fs::File::open(&path).expect("load program");
        f.read_to_end(&mut buf).expect("read program");
        Bytes::from(buf.to_vec())
    };
    pub static ref CHALLENGE_LOCK_CODE_HASH: [u8; 32] = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&CHALLENGE_LOCK_PROGRAM);
        hasher.finalize(&mut buf);
        buf
    };
    pub static ref STATE_VALIDATOR_PROGRAM: Bytes = {
        let mut buf = Vec::new();
        let mut path = PathBuf::new();
        path.push(&SCRIPT_DIR);
        path.push(&STATE_VALIDATOR);
        let mut f = fs::File::open(&path).expect("load program");
        f.read_to_end(&mut buf).expect("read program");
        Bytes::from(buf.to_vec())
    };
    pub static ref STATE_VALIDATOR_CODE_HASH: [u8; 32] = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&STATE_VALIDATOR_PROGRAM);
        hasher.finalize(&mut buf);
        buf
    };
    pub static ref ETH_ACCOUNT_LOCK_PROGRAM: Bytes = {
        let mut buf = Vec::new();
        let mut path = PathBuf::new();
        path.push(&SCRIPT_DIR);
        path.push(&ETH_LOCK_PATH);
        let mut f = fs::File::open(&path).expect("load program");
        f.read_to_end(&mut buf).expect("read program");
        Bytes::from(buf.to_vec())
    };
    pub static ref ETH_ACCOUNT_LOCK_CODE_HASH: [u8; 32] = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&ETH_ACCOUNT_LOCK_PROGRAM);
        hasher.finalize(&mut buf);
        buf
    };
    pub static ref TRON_ACCOUNT_LOCK_PROGRAM: Bytes = {
        let mut buf = Vec::new();
        let mut path = PathBuf::new();
        path.push(&SCRIPT_DIR);
        path.push(&TRON_LOCK_PATH);
        let mut f = fs::File::open(&path).expect("load program");
        f.read_to_end(&mut buf).expect("read program");
        Bytes::from(buf.to_vec())
    };
    pub static ref TRON_ACCOUNT_LOCK_CODE_HASH: [u8; 32] = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&TRON_ACCOUNT_LOCK_PROGRAM);
        hasher.finalize(&mut buf);
        buf
    };
    pub static ref SECP256K1_DATA: Bytes = {
        let mut buf = Vec::new();
        let mut f = fs::File::open(&SECP256K1_DATA_PATH).expect("load secp256k1 data");
        f.read_to_end(&mut buf).expect("read secp256k1 data");
        Bytes::from(buf.to_vec())
    };
    pub static ref SECP256K1_DATA_HASH: [u8; 32] = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&SECP256K1_DATA);
        hasher.finalize(&mut buf);
        buf
    };
    pub static ref META_CONTRACT_VALIDATOR_PROGRAM: Bytes = {
        let mut buf = Vec::new();
        let mut path = PathBuf::new();
        path.push(&C_SCRIPTS_DIR);
        path.push(&META_CONTRACT_BIN_NAME);
        let mut f = fs::File::open(&path).expect("load program");
        f.read_to_end(&mut buf).expect("read program");
        Bytes::from(buf.to_vec())
    };
    pub static ref META_CONTRACT_CODE_HASH: [u8; 32] = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&META_CONTRACT_VALIDATOR_PROGRAM);
        hasher.finalize(&mut buf);
        buf
    };
    pub static ref WITHDRAWAL_LOCK_PROGRAM: Bytes = {
        let mut buf = Vec::new();
        let mut path = PathBuf::new();
        path.push(&SCRIPT_DIR);
        path.push(&WITHDRAWAL_LOCK_PATH);
        let mut f = fs::File::open(&path).expect("load withdrawal lock program");
        f.read_to_end(&mut buf)
            .expect("read withdrawal lock program");
        Bytes::from(buf.to_vec())
    };
    pub static ref WITHDRAWAL_LOCK_CODE_HASH: [u8; 32] = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&WITHDRAWAL_LOCK_PROGRAM);
        hasher.finalize(&mut buf);
        buf
    };
}
