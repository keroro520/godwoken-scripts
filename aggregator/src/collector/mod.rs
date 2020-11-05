//! Collector trait and implementations

pub mod error;
pub mod lumos;

use ckb_types::packed::{Header, Transaction};
use error::Error;
use gw_jsonrpc_types::collector::QueryParam;

pub struct TransactionInfo {
    pub transaction: Transaction,
    pub tx_hash: [u8; 32],
    pub block_hash: [u8; 32],
    pub status: String,
}

pub trait Collector {
    fn subscribe(&self, param: QueryParam, callback: String) -> Result<(), Error>;
    fn query_transactions(&self, param: QueryParam) -> Result<Vec<TransactionInfo>, Error>;
    fn get_transaction(&self, tx_hash: &[u8; 32]) -> Result<TransactionInfo, Error>;
    fn get_header(&self, block_hash: &[u8; 32]) -> Result<Option<Header>, Error>;
    fn get_header_by_number(&self, number: u64) -> Result<Option<Header>, Error>;
}
