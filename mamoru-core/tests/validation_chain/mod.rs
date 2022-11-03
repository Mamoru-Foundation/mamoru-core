use cosmrs::crypto::{secp256k1, PublicKey};
use cosmrs::{AccountId, Coin};
use mamoru_core::validation_chain::{
    AccountConfig, ConnectionConfig, MessageClient, MessageClientConfig, QueryClient,
    QueryClientConfig,
};
use serde::Serialize;
use std::error::Error;

mod message_client;
mod query_client;
mod sniffer;

/// Bech32 prefix for an account
const ACCOUNT_PREFIX: &str = "cosmos";

/// Denom name
const DENOM: &str = "token";

const FAUCET_ENDPOINT_ENV: &str = "MAMORU_TEST_FAUCET_ENDPOINT";

struct TestAccount {
    key: secp256k1::SigningKey,
}

impl TestAccount {
    fn new() -> Self {
        Self {
            key: secp256k1::SigningKey::random(),
        }
    }

    async fn with_faucet() -> Self {
        let account = Self::new();
        account.faucet().await.unwrap();

        account
    }

    async fn faucet(&self) -> Result<(), Box<dyn Error>> {
        let faucet_endpoint = std::env::var(FAUCET_ENDPOINT_ENV)
            .unwrap_or_else(|_| panic!("Missing {} env var", FAUCET_ENDPOINT_ENV));

        let request = FaucetRequest {
            address: self.address().to_string(),
            coins: vec![coin(100).to_string()],
        };

        let client = reqwest::Client::new();
        client.post(faucet_endpoint).json(&request).send().await?;

        Ok(())
    }

    fn public_key(&self) -> PublicKey {
        self.key.public_key()
    }

    fn address(&self) -> AccountId {
        self.public_key()
            .account_id(ACCOUNT_PREFIX)
            .expect("Never fails with `secp256k1` key.")
    }
}

#[derive(Serialize)]
struct FaucetRequest {
    address: String,
    coins: Vec<String>,
}

fn coin(amount: u128) -> Coin {
    Coin {
        amount,
        denom: DENOM.parse().unwrap(),
    }
}

async fn message_client() -> MessageClient {
    let TestAccount { key } = TestAccount::with_faucet().await;

    MessageClient::connect(MessageClientConfig {
        connection: ConnectionConfig::from_env(),
        chain: Default::default(),
        account: AccountConfig::new(key),
    })
    .await
    .expect("Connection error")
}

async fn query_client() -> QueryClient {
    QueryClient::connect(QueryClientConfig {
        connection: ConnectionConfig::from_env(),
    })
    .await
    .expect("Connection error")
}
