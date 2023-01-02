use blockchain_data_macro::BlockchainData;
use datafusion::arrow::datatypes::DataType;
use mamoru_core::ValueData;

#[derive(BlockchainData)]
#[schema(table_name = "call_traces")]
pub struct CallTrace {
    #[schema(type = "DataType::UInt64")]
    pub seq: u64,

    #[schema(type = "DataType::UInt64")]
    pub tx_seq: u64,

    #[schema(type = "DataType::UInt32")]
    pub depth: u32,

    #[schema(type = "DataType::UInt8")]
    pub call_type: u8,

    #[schema(type = "DataType::UInt64")]
    pub gas_used: u64,

    #[schema(type = "DataType::Utf8", nullable = true)]
    pub transaction_module: Option<String>,

    #[schema(type = "DataType::Utf8")]
    pub function: String,
}

#[derive(BlockchainData)]
#[schema(table_name = "call_trace_type_args")]
pub struct CallTraceTypeArg {
    #[schema(type = "DataType::UInt64")]
    pub seq: u64,

    #[schema(type = "DataType::UInt64")]
    pub call_trace_seq: u64,

    #[schema(type = "DataType::Utf8")]
    pub arg: String,
}

#[derive(BlockchainData)]
#[schema(table_name = "call_trace_args")]
pub struct CallTraceArg {
    #[schema(type = "DataType::UInt64")]
    pub seq: u64,

    #[schema(type = "DataType::UInt64")]
    pub call_trace_seq: u64,

    #[schema(type = "DataType::Binary")]
    pub arg: ValueData,
}
