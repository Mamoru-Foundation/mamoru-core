use crate::blockchain_data::value::{StructValue, Value, ValueData};
use crate::{BlockchainDataCtx, BlockchainDataCtxBuilder};
use blockchain_data_macro::BlockchainData;
use datafusion::arrow::datatypes::{DataType, TimeUnit};
use maplit::hashmap;

pub fn data_ctx(tx_hash: impl Into<String>) -> BlockchainDataCtx {
    let (tx_seq, digest) = (42, tx_hash.into());
    let now = chrono::Utc::now().naive_utc();

    let builder = BlockchainDataCtxBuilder::new();

    builder
        .add_data(
            TestTransactionBatch::new(vec![TestTransaction {
                seq: tx_seq,
                time: now.timestamp(),
                digest: digest.clone(),
                gas_used: 42_000,
            }])
            .boxed(),
        )
        .unwrap();

    builder
        .add_data(
            TestCallTraceBatch::new(vec![
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
            ])
            .boxed(),
        )
        .unwrap();

    builder
        .add_data(
            TestCallTraceArgBatch::new(vec![
                TestCallTraceArg {
                    seq: 0,
                    call_trace_seq: 1,
                    arg: ValueData::new(Value::U64(42)).unwrap(),
                },
                TestCallTraceArg {
                    seq: 1,
                    call_trace_seq: 1,
                    arg: ValueData::new(Value::Struct(StructValue::new(
                        "dummy".to_string(),
                        hashmap! {
                             "foo".to_string() => Value::U64(42),
                             "bar".to_string() => Value::String("hello".to_string()),
                        },
                    )))
                    .unwrap(),
                },
                TestCallTraceArg {
                    seq: 2,
                    call_trace_seq: 1,
                    arg: ValueData::new(Value::Bool(false)).unwrap(),
                },
            ])
            .boxed(),
        )
        .unwrap();

    builder.finish(format!("{}", tx_seq), digest, now)
}

#[derive(BlockchainData)]
#[schema(table_name = "transactions", mamoru_path = "crate")]
pub struct TestTransaction {
    #[schema(type = "DataType::UInt64")]
    pub seq: u64,

    #[schema(type = "DataType::Utf8")]
    pub digest: String,

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

#[derive(BlockchainData)]
#[schema(table_name = "call_trace_args", mamoru_path = "crate")]
pub struct TestCallTraceArg {
    #[schema(type = "DataType::UInt64")]
    pub seq: u64,

    #[schema(type = "DataType::UInt64")]
    pub call_trace_seq: u64,

    #[schema(type = "DataType::Binary")]
    pub arg: ValueData,
}
