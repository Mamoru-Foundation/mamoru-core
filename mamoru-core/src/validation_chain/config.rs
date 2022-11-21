use crate::errors::ValidationClientError;
use crate::from_env;
use crate::validation_chain::ClientResult;
use cosmrs::crypto::{secp256k1, PublicKey};
use cosmrs::tendermint::chain;
use cosmrs::{AccountId, Denom};
use serde::{Deserialize, Deserializer};
use std::sync::Arc;

/// The configuration required for creating MessageClient
#[derive(Deserialize, Clone)]
pub struct MessageClientConfig {
    #[serde(flatten)]
    pub connection: ConnectionConfig,

    #[serde(default)]
    pub chain: ChainConfig,

    #[serde(flatten)]
    pub account: AccountConfig,
}

impl MessageClientConfig {
    pub fn from_env() -> Self {
        from_env()
    }

    pub fn public_key(&self) -> PublicKey {
        self.private_key().public_key()
    }

    pub fn private_key(&self) -> &secp256k1::SigningKey {
        &self.account.private_key
    }

    pub fn address(&self) -> AccountId {
        self.public_key()
            .account_id(&self.chain.account_id_prefix)
            .expect("Never fails with `secp256k1` key.")
    }

    pub fn chain_id(&self) -> ClientResult<chain::Id> {
        self.chain
            .chain_id
            .clone()
            .parse()
            .map_err(ValidationClientError::ParseChainId)
    }

    pub fn account_id_prefix(&self) -> &str {
        &self.chain.account_id_prefix
    }

    pub fn token_denominator(&self) -> ClientResult<Denom> {
        self.chain
            .token_denominator
            .as_str()
            .parse()
            .map_err(ValidationClientError::ParseTokenDenominator)
    }

    pub fn tx_gas_limit(&self) -> u64 {
        self.chain.tx_gas_limit
    }

    pub fn tx_fee_amount(&self) -> u128 {
        self.chain.tx_fee_amount
    }
}

/// The configuration required for creating QueryClient
#[derive(Deserialize, Clone, Debug)]
pub struct QueryClientConfig {
    #[serde(flatten)]
    pub connection: ConnectionConfig,
}

impl QueryClientConfig {
    pub fn from_env() -> Self {
        from_env()
    }
}

/// Connection parameters for the Validation Chain API
#[derive(Deserialize, Clone, Debug)]
pub struct ConnectionConfig {
    pub endpoint: String,
}

impl ConnectionConfig {
    pub fn from_env() -> Self {
        from_env()
    }
}

/// Cosmos chain-specific configuration
#[derive(Deserialize, Clone, Debug)]
pub struct ChainConfig {
    pub chain_id: String,
    pub account_id_prefix: String,
    pub token_denominator: String,
    pub tx_gas_limit: u64,
    pub tx_fee_amount: u128,
}

impl Default for ChainConfig {
    fn default() -> Self {
        ChainConfig {
            chain_id: "validationchain".to_string(),
            account_id_prefix: "cosmos".to_string(),
            token_denominator: "token".to_string(),
            tx_gas_limit: 100_000,
            tx_fee_amount: 0,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct AccountConfig {
    #[serde(deserialize_with = "key_from_base64_bytes")]
    pub private_key: Arc<secp256k1::SigningKey>,
}

impl AccountConfig {
    pub fn new(private_key: secp256k1::SigningKey) -> Self {
        Self {
            private_key: Arc::new(private_key),
        }
    }
}

fn key_from_base64_bytes<'de, D>(deserializer: D) -> Result<Arc<secp256k1::SigningKey>, D::Error>
where
    D: Deserializer<'de>,
{
    let base64_string: String = Deserialize::deserialize(deserializer)?;
    let bytes = base64::decode(base64_string).expect("Can not parse private key base64");
    let key = secp256k1::SigningKey::from_bytes(&bytes).expect("Can not parse private key bytes");

    Ok(Arc::new(key))
}
