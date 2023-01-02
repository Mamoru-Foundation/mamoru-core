use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Failed to create RecordBatch for the internal database.")]
    CreateRecordBatch(datafusion::arrow::error::ArrowError),

    #[error("Failed to covert RecordBatch to json data.")]
    RecordBatchToJson(datafusion::arrow::error::ArrowError),

    #[error("Failed to register RecordBatch in the internal database.")]
    RegisterRecordBatch(datafusion::error::DataFusionError),

    #[error("Failed to parse SQL.")]
    ParseSql(datafusion::sql::sqlparser::parser::ParserError),

    #[error("Failed to plan an already parsed query.")]
    PlanQuery(datafusion::error::DataFusionError),

    #[error("Failed to execute query.")]
    ExecuteQuery(datafusion::error::DataFusionError),

    #[error("We only support a single SQL statement.")]
    WrongStatementsNumber,

    #[error("We only support SELECT statements. Queries like INSERT, CREATE TABLE, etc are not allowed.")]
    UnsupportedStatement,
}

#[derive(Error, Debug)]
pub enum ValueError {
    #[error("Failed to serialize the value.")]
    Serialize(Box<dyn Error>),

    #[error("Failed to deserialize the value.")]
    Deserialize(Box<dyn Error>),
}
