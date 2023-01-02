mod call_trace;
mod events;
mod transaction;

pub use call_trace::*;
pub use events::*;
pub use transaction::*;

use blockchain_data_macro::tables;
use datafusion::arrow::error::ArrowError;
use mamoru_core::{BlockchainData, TableDef};

pub fn all_tables() -> Result<Vec<TableDef>, ArrowError> {
    tables![
        Transaction,
        CallTrace,
        CallTraceTypeArg,
        CallTraceArg,
        MoveEvent,
        PublishEvent,
        CoinBalanceChangeEvent,
        EpochChangeEvent,
        CheckpointEvent,
        TransferObjectEvent,
        MutateObjectEvent,
        DeleteObjectEvent,
        NewObjectEvent,
    ]
}
