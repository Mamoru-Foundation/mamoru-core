use crate::{BlockchainDataCtx, BlockchainDataCtxBuilder};
use blockchain_data_macro::BlockchainData;
use datafusion::arrow::datatypes::{DataType, TimeUnit};

pub fn data_ctx(tx_hash: impl Into<String>) -> BlockchainDataCtx {
    let (tx_seq, digest) = (42, tx_hash.into());
    let now = chrono::Utc::now().naive_utc();

    let builder = BlockchainDataCtxBuilder::new();

    builder
        .add_data(TestTransactionBatch::new(vec![TestTransaction {
            seq: tx_seq,
            time: now.timestamp(),
            hash: digest.clone(),
            gas_used: 42_000,
        }]))
        .unwrap();

    builder
        .add_data(TestCallTraceBatch::new(vec![
            TestCallTrace {
                seq: 0,
                tx_seq,
                function: "func1".to_string(),
            },
            TestCallTrace {
                seq: 1,
                tx_seq,
                function: "func2".to_string(),
            },
        ]))
        .unwrap();

    builder.finish(format!("{}", tx_seq), digest, now)
}

#[derive(BlockchainData)]
#[schema(table_name = "transactions", mamoru_path = "crate")]
pub struct TestTransaction {
    #[schema(type = "DataType::UInt64")]
    pub seq: u64,

    #[schema(type = "DataType::Utf8")]
    pub hash: String,

    #[schema(type = "DataType::Timestamp(TimeUnit::Second, None)")]
    pub time: i64,

    #[schema(type = "DataType::UInt64")]
    pub gas_used: u64,
}

#[derive(BlockchainData)]
#[schema(table_name = "call_traces", mamoru_path = "crate")]
pub struct TestCallTrace {
    #[schema(type = "DataType::UInt64")]
    pub seq: u64,

    #[schema(type = "DataType::UInt64")]
    pub tx_seq: u64,

    #[schema(type = "DataType::Utf8")]
    pub function: String,
}
