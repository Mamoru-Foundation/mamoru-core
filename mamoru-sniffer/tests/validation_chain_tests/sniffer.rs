use crate::validation_chain_tests::{message_client, query_client, sniffer};
use chrono::{Days, Utc};
use futures::TryStreamExt;
use mamoru_core::test_blockchain_data::data_ctx;
use mamoru_sniffer::validation_chain::{ChainType, IncidentQueryResponseDto};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::future;
use test_log::test;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::{Action, Retry};

#[test(tokio::test)]
#[ignore]
async fn smoke() {
    let chain = ChainType::SuiDevnet;
    let tx_hash = random_string();
    let rule_id = random_string();

    register_active_rule(
        &rule_id,
        chain,
        format!(
            "SELECT 1 FROM transactions t WHERE t.digest = '{}'",
            &tx_hash
        ),
    )
    .await;

    let sniffer = sniffer(chain).await;

    let incidents = get_incidents(&rule_id).await;
    assert!(incidents.is_empty());

    sniffer
        .observe_data(data_ctx(tx_hash.clone()))
        .await
        .expect("Failed to observe data.");

    retry(|| async {
        let incidents = get_incidents(&rule_id).await;

        if incidents.len() == 1 {
            Ok(())
        } else {
            Err(format!("Wrong incidents len: {}", incidents.len()))
        }
    })
    .await
    .expect("Failed to query incidents.");
}

async fn retry<A>(action: A) -> Result<<A as Action>::Item, <A as Action>::Error>
where
    A: Action,
{
    Retry::spawn(FixedInterval::from_millis(1000).take(10), action).await
}

async fn get_incidents(rule_id: &str) -> Vec<IncidentQueryResponseDto> {
    query_client()
        .await
        .list_incidents()
        .try_filter(|i| future::ready(i.rule_id == rule_id))
        .try_collect::<Vec<_>>()
        .await
        .expect("List incidents error")
}

async fn register_active_rule(
    rule_id: impl Into<String>,
    chain: ChainType,
    query: impl Into<String>,
) {
    let message_client = message_client().await;

    message_client
        .register_rule(
            rule_id,
            chain,
            query,
            Utc::now(),
            Utc::now().checked_add_days(Days::new(1)).unwrap(),
        )
        .await
        .expect("Register rule error");
}

fn random_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect()
}
