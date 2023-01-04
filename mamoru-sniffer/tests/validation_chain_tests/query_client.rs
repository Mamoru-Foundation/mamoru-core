use crate::validation_chain_tests::query_client;
use futures::TryStreamExt;
use mamoru_sniffer::validation_chain::ChainType;
use test_log::test;

#[test(tokio::test)]
#[ignore]
async fn smoke() {
    let client = query_client().await;

    let _ = client
        .list_rules(ChainType::SuiDevnet)
        .try_collect::<Vec<_>>()
        .await
        .expect("List rules error");

    let _ = client
        .list_sniffers()
        .try_collect::<Vec<_>>()
        .await
        .expect("List sniffers error");

    let _ = client
        .list_incidents()
        .try_collect::<Vec<_>>()
        .await
        .expect("List incidents error");
}
