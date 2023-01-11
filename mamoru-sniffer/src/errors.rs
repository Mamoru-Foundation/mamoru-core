use cosmrs::proto::cosmos::base::abci::v1beta1::TxResponse;
use cosmrs::{tendermint, ErrorReport};
use mamoru_core::DataError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SnifferError {
    #[error(transparent)]
    Client(#[from] ValidationClientError),

    #[error("Failed to parse Rule")]
    RuleParse(#[from] RuleParseError),

    #[error("Failed to parse Config")]
    Config(#[from] envy::Error),
}

#[derive(Error, Debug)]
pub enum RuleParseError {
    #[error("Invalid DateTime format")]
    DateTime(#[source] chrono::ParseError),

    #[error(transparent)]
    DataError(#[from] DataError),
}

#[derive(Error, Debug)]
pub enum ValidationClientError {
    #[error("Failed to connect to the Validation Chain")]
    Connect(#[from] tonic::transport::Error),

    #[error("Failed to call the Validation Chain API")]
    Request(#[from] tonic::Status),

    #[error("Failed to encode protobuf message")]
    EncodeMessage(#[from] prost::EncodeError),

    #[error("Failed to decode protobuf message")]
    DecodeMessage(#[from] prost::DecodeError),

    #[error("Failed to take transaction as bytes")]
    TransactionToBytes(#[source] ErrorReport),

    #[error("Failed to create Sign Doc")]
    CreateSignDoc(#[source] ErrorReport),

    #[error("Failed to sign a transaction")]
    SignTransaction(#[source] ErrorReport),

    #[error("Failed to parse token denominator")]
    ParseTokenDenominator(#[source] ErrorReport),

    #[error("Failed to parse chain id")]
    ParseChainId(#[source] tendermint::Error),

    #[error("Validation chain returned an error")]
    CosmosSdkError(#[from] CosmosSdkError),
}

impl ValidationClientError {
    pub(crate) fn is_incorrect_account_sequence(&self) -> bool {
        matches!(
            self,
            Self::CosmosSdkError(CosmosSdkError::IncorrectAccountSequence)
        )
    }
}

#[derive(Error, Debug)]
pub enum CosmosSdkError {
    #[error("The account sequence defined in the signer info doesn't match the account's actual sequence number")]
    IncorrectAccountSequence,

    #[error("Cannot parse a transaction: {raw_log}")]
    TxDecode { raw_log: String },

    #[error("Invalid request: {raw_log}")]
    InvalidRequest { raw_log: String },

    #[error("Unknown error, code: {code}, raw_log: {raw_log}")]
    Other { code: u32, raw_log: String },
}

impl TryFrom<TxResponse> for CosmosSdkError {
    type Error = ();

    fn try_from(value: TxResponse) -> Result<Self, Self::Error> {
        match value.code {
            // code 0 is success
            0 => Err(()),
            // See https://github.com/cosmos/cosmos-sdk/blob/main/types/errors/errors.go
            // for more error codes
            2 => Ok(CosmosSdkError::TxDecode {
                raw_log: value.raw_log,
            }),
            18 => Ok(CosmosSdkError::InvalidRequest {
                raw_log: value.raw_log,
            }),
            32 => Ok(CosmosSdkError::IncorrectAccountSequence),
            code => Ok(CosmosSdkError::Other {
                code,
                raw_log: value.raw_log,
            }),
        }
    }
}
