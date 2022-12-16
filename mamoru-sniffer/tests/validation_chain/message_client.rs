use crate::validation_chain::message_client;
use mamoru_sniffer::validation_chain::{
    BlockId, ChainType, IncidentReport, IncidentSource, TransactionId,
};
use test_log::test;

#[test(tokio::test)]
#[ignore]
async fn smoke() {
    let client = message_client().await;
    let rule_ids = vec!["test_rule_id0".to_string(), "test_rule_id1".to_string()];

    client
        .register_sniffer(ChainType::AptosMainnet)
        .await
        .expect("Register sniffer error");

    client
        .subscribe_rules(rule_ids.clone())
        .await
        .expect("Subscribe rules error");

    let incidents: Vec<_> = rule_ids
        .into_iter()
        .map(|rule_id| IncidentReport {
            rule_id,
            source: IncidentSource::Transaction {
                transaction: TransactionId {
                    tx_id: "test_tx_id".to_string(),
                    hash: "test_tx_id".to_string(),
                },
                block: Some(BlockId {
                    block_id: "test_block_id".to_string(),
                    hash: "test_block_id".to_string(),
                }),
            },
        })
        .collect();

    client
        .report_incidents(incidents)
        .await
        .expect("Report incidents error");

    client
        .unregister_sniffer()
        .await
        .expect("Unregister sniffer error");
}
