use blockchain_data_macro::BlockchainData;
use datafusion::arrow::datatypes::DataType;

#[derive(BlockchainData)]
#[schema(table_name = "transactions")]
pub struct Transaction {
    #[schema(type = "DataType::UInt64")]
    pub index: u64,

    #[schema(type = "DataType::UInt64")]
    pub block_index: u64,

    #[schema(type = "DataType::Binary")]
    pub hash: Vec<u8>,

    #[schema(type = "DataType::UInt8", rename = "type")]
    pub typ: u8,

    #[schema(type = "DataType::UInt32")]
    pub nonce: u32,

    #[schema(type = "DataType::Utf8")]
    pub status: String,

    #[schema(type = "DataType::UInt32")]
    pub timestamp: u32,

    #[schema(type = "DataType::Binary")]
    pub from: Vec<u8>,

    #[schema(type = "DataType::Binary")]
    pub to: Vec<u8>,
}
