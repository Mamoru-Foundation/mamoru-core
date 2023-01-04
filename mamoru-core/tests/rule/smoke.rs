use crate::rule::{active_rule, inactive_rule};
use mamoru_core::test_blockchain_data::data_ctx;
use mamoru_core::{DataError, Rule};
use serde_json::json;
use test_log::test;

#[test(tokio::test)]
async fn simple_query_matches() -> Result<(), DataError> {
    let ctx = data_ctx("DUMMY_HASH");
    let rule = active_rule("SELECT t.seq FROM transactions t WHERE t.digest = 'DUMMY_HASH'");

    let data = rule.verify(&ctx).await?;

    let row = data.outputs.get(0).expect("At least one row returned.");
    let seq_field = row.get("seq").cloned().expect("seq field exists.");

    assert!(data.matched);
    assert_eq!(seq_field, json!(42));

    Ok(())
}

#[test(tokio::test)]
async fn inactive_rule_doesnt_match() -> Result<(), DataError> {
    let ctx = data_ctx("DUMMY_HASH");
    let rule = inactive_rule("SELECT t.seq FROM transactions t WHERE t.hash = 'DUMMY_HASH'");

    let data = rule.verify(&ctx).await?;

    assert!(!data.matched);
    assert!(data.outputs.is_empty());

    Ok(())
}

#[test(tokio::test)]
async fn outputs_may_consist_few_rows() -> Result<(), DataError> {
    let ctx = data_ctx("DUMMY_HASH");
    let rule = active_rule("SELECT ct.function FROM call_traces ct");

    let data = rule.verify(&ctx).await?;

    assert!(data.matched);
    assert_eq!(data.outputs.len(), 2);

    Ok(())
}

#[test(tokio::test)]
async fn cte_expressions_are_supported() -> Result<(), DataError> {
    let ctx = data_ctx("DUMMY_HASH");
    let rule = active_rule(
        r#"
        WITH func1_traces AS (
            SELECT * FROM call_traces ct WHERE ct.function = 'func1'
        )

        SELECT * FROM transactions t
            INNER JOIN func1_traces ct ON ct.seq IS NOT NULL
    "#,
    );

    let data = rule.verify(&ctx).await?;

    assert!(data.matched);

    Ok(())
}

#[test(tokio::test)]
async fn write_expressions_are_not_allowed() -> Result<(), DataError> {
    let statements = [
        "INSERT INTO call_traces VALUES (3, 4, 'func3')",
        "CREATE EXTERNAL TABLE foo STORED AS CSV LOCATION 'server_passwords.csv'",
    ];

    for statement in statements {
        let rule = Rule::new("dummy".to_string(), 0, i64::MAX, statement);

        assert!(matches!(rule, Err(DataError::UnsupportedStatement)));
    }

    Ok(())
}
