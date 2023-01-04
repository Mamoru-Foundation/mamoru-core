pub use crate::validation_chain::proto::validation_chain::{
    chain::ChainType, Block as BlockId, Transaction as TransactionId,
};

use crate::errors::ValidationClientError;
use crate::validation_chain::config::MessageClientConfig;
use crate::validation_chain::proto::cosmos::auth::v1beta1::query_client::QueryClient as CosmosQueryClient;
use crate::validation_chain::proto::cosmos::auth::v1beta1::{BaseAccount, QueryAccountRequest};
use crate::validation_chain::proto::cosmos::tx::v1beta1::service_client::ServiceClient as CosmosServiceClient;
use crate::validation_chain::proto::cosmos::tx::v1beta1::{BroadcastMode, BroadcastTxRequest};
use crate::validation_chain::proto::validation_chain::source::SourceType;
use crate::validation_chain::proto::validation_chain::{
    Chain, IncidentReportCommandRequestDto, MsgRegisterRule, MsgRegisterSniffer, MsgReportIncident,
    MsgSubscribeRules, MsgUnregisterSniffer, RuleRegisterCommandRequestDto,
    RulesSubscribeCommandRequestDto, SnifferRegisterCommandRequestDto,
    SnifferUnregisterCommandRequestDto, Source,
};
use crate::validation_chain::ClientResult;
use chrono::Utc;
use cosmrs::proto::traits::TypeUrl;
use cosmrs::tx::{
    AccountNumber, Body, BodyBuilder, Fee, MessageExt, SequenceNumber, SignDoc, SignerInfo,
};
use cosmrs::{AccountId, Coin};
use prost::Message;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::error;

const MAX_RETRIES: usize = 5;
const RETRY_SLEEP_TIME: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct IncidentReport {
    pub rule_id: String,
    pub source: IncidentSource,
}

/// Safer wrapper over part of [`IncidentReportCommandRequestDto`]
#[derive(Debug)]
pub enum IncidentSource {
    Mempool,
    Transaction {
        // Sui doesn't have blocks at all
        block: Option<BlockId>,
        transaction: TransactionId,
    },
}

/// Cache for account data, that is required to specify
/// during transaction signing.
#[derive(Clone, Copy)]
struct AccountDataCache {
    number: AccountNumber,
    sequence: SequenceNumber,
}

impl AccountDataCache {
    /// Loads [`Self`] via Cosmos API
    async fn fetch(
        mut query_client: CosmosQueryClient<tonic::transport::Channel>,
        address: AccountId,
    ) -> ClientResult<Self> {
        let response = query_client
            .account(QueryAccountRequest {
                address: address.to_string(),
            })
            .await?
            .into_inner();

        let account = BaseAccount::decode(&*response.account.expect("Always exists.").value)?;

        Ok(Self {
            number: account.account_number,
            sequence: account.sequence,
        })
    }
}

/// High-level wrapper for incident-reporting to Validation Chain.
#[derive(Clone)]
pub struct MessageClient {
    account_data: Arc<Mutex<AccountDataCache>>,
    config: MessageClientConfig,
    service_client: CosmosServiceClient<tonic::transport::Channel>,
    query_client: CosmosQueryClient<tonic::transport::Channel>,
}

impl MessageClient {
    /// Connects to the Validation Chain.
    /// Call [`MessageClientConfig::from_env`] to create `config` parameter from environment variables
    pub async fn connect(config: MessageClientConfig) -> ClientResult<Self> {
        let service_client =
            CosmosServiceClient::connect(config.connection.endpoint.clone()).await?;
        let query_client = CosmosQueryClient::connect(config.connection.endpoint.clone()).await?;
        let account_data = AccountDataCache::fetch(query_client.clone(), config.address()).await?;

        Ok(Self {
            account_data: Arc::new(Mutex::new(account_data)),
            config,
            service_client,
            query_client,
        })
    }

    pub async fn register_sniffer(&self, chain: ChainType) -> ClientResult<()> {
        let sniffer = self.config.address().to_string();

        self.sign_and_broadcast_txs(vec![MsgRegisterSniffer {
            creator: sniffer.clone(),
            sniffer: Some(SnifferRegisterCommandRequestDto {
                sniffer,
                chain: Some(Chain {
                    chain_type: chain.into(),
                }),
            }),
        }])
        .await?;

        Ok(())
    }

    pub async fn unregister_sniffer(&self) -> ClientResult<()> {
        let sniffer = self.config.address().to_string();

        self.sign_and_broadcast_txs(vec![MsgUnregisterSniffer {
            creator: sniffer.clone(),
            sniffer: Some(SnifferUnregisterCommandRequestDto { sniffer }),
        }])
        .await?;

        Ok(())
    }

    pub async fn subscribe_rules(&self, rule_ids: Vec<String>) -> ClientResult<()> {
        let sniffer = self.config.address().to_string();

        self.sign_and_broadcast_txs(vec![MsgSubscribeRules {
            creator: sniffer.clone(),
            rules: Some(RulesSubscribeCommandRequestDto { rule_ids, sniffer }),
        }])
        .await?;

        Ok(())
    }

    pub async fn report_incidents(
        &self,
        reports: impl IntoIterator<Item = IncidentReport>,
    ) -> ClientResult<()> {
        let sniffer = self.config.address().to_string();

        let report_messages: Vec<_> = reports
            .into_iter()
            .map(|report| {
                let (source, block, tx) = match report.source {
                    IncidentSource::Mempool => (SourceType::Mempool, None, None),
                    IncidentSource::Transaction { block, transaction } => {
                        (SourceType::Block, block, Some(transaction))
                    }
                };

                MsgReportIncident {
                    creator: sniffer.clone(),
                    incident: Some(IncidentReportCommandRequestDto {
                        sniffer: sniffer.clone(),
                        source: Some(Source {
                            source_type: source.into(),
                        }),
                        rule_id: report.rule_id,
                        block,
                        tx,
                    }),
                }
            })
            .collect();

        self.sign_and_broadcast_txs(report_messages).await?;

        Ok(())
    }

    pub async fn register_rule(
        &self,
        rule_id: impl Into<String>,
        chain: ChainType,
        content: impl Into<String>,
        activate_since: chrono::DateTime<Utc>,
        inactivate_since: chrono::DateTime<Utc>,
    ) -> ClientResult<()> {
        let sniffer = self.config.address().to_string();

        self.sign_and_broadcast_txs(vec![MsgRegisterRule {
            creator: sniffer,
            rule: Some(RuleRegisterCommandRequestDto {
                rule_id: rule_id.into(),
                chain: Some(Chain {
                    chain_type: chain.into(),
                }),
                ipfs_cid: "unknown".to_string(),
                content: content.into(),
                activate_since: activate_since.to_rfc3339(),
                inactivate_since: inactivate_since.to_rfc3339(),
            }),
        }])
        .await?;

        Ok(())
    }

    /// Unlike queries, commands in Cosmos must be signed and published as transactions.
    /// This method handles transaction signing and ordering ( like setting `sequence_number`).
    async fn sign_and_broadcast_txs<T: Message + TypeUrl>(
        &self,
        messages: impl IntoIterator<Item = T>,
    ) -> ClientResult<()> {
        let mut builder = BodyBuilder::new();

        for message in messages.into_iter() {
            builder.msg(message.to_any()?);
        }

        let tx_body = builder.finish();

        let mut account_data = self.account_data.lock().await;

        for _ in 0..MAX_RETRIES {
            match self
                .sign_and_broadcast_tx_impl(tx_body.clone(), *account_data)
                .await
            {
                Ok(_) => break,
                Err(err) => {
                    if err.is_incorrect_account_sequence() {
                        *account_data = AccountDataCache::fetch(
                            self.query_client.clone(),
                            self.config.address(),
                        )
                        .await?;
                    } else {
                        error!("Got unknown error from validation chain: {:?}", err);

                        return Err(err);
                    }
                }
            }

            tokio::time::sleep(RETRY_SLEEP_TIME).await;
        }

        // Assume our app is the only account user
        // Intended to reduce API call rate
        account_data.sequence += 1;

        Ok(())
    }

    async fn sign_and_broadcast_tx_impl(
        &self,
        tx_body: Body,
        account_data: AccountDataCache,
    ) -> ClientResult<()> {
        let AccountDataCache { number, sequence } = account_data;
        let auth_info = SignerInfo::single_direct(Some(self.config.public_key()), sequence)
            .auth_info(Fee::from_amount_and_gas(
                Coin {
                    denom: self.config.token_denominator()?,
                    amount: self.config.tx_fee_amount(),
                },
                self.config.tx_gas_limit(),
            ));

        let chain_id = self.config.chain_id()?;

        let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, number)
            .map_err(ValidationClientError::CreateSignDoc)?;

        let tx_raw = sign_doc
            .sign(self.config.private_key())
            .map_err(ValidationClientError::SignTransaction)?;

        let response = self
            .service_client
            .clone()
            .broadcast_tx(BroadcastTxRequest {
                tx_bytes: tx_raw
                    .to_bytes()
                    .map_err(ValidationClientError::TransactionToBytes)?,
                mode: BroadcastMode::Sync.into(),
            })
            .await?
            .into_inner();

        let tx_response = response.tx_response.expect("Always exists.");

        match tx_response.try_into().ok() {
            // If code is an error code, return proper error
            Some(error) => Err(ValidationClientError::CosmosSdkError(error)),
            // Ok otherwise
            None => Ok(()),
        }
    }
}
