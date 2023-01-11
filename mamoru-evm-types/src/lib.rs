mod transaction;

pub use transaction::*;

use blockchain_data_macro::tables;
use datafusion::arrow::error::ArrowError;
use mamoru_core::{BlockchainData, TableDef};

pub fn all_tables() -> Result<Vec<TableDef>, ArrowError> {
    tables![Transaction,]
}
