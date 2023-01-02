use crate::rule::active_rule;
use mamoru_core::test_blockchain_data::data_ctx;
use mamoru_core::DataError;
use test_log::test;

#[test(tokio::test)]
async fn uint64_type_conversion() -> Result<(), DataError> {
    let ctx = data_ctx("DUMMY_HASH");
    let rule = active_rule("SELECT 1 FROM call_trace_args cta WHERE as_uint64(cta.arg) = 42;");

    let data = rule.verify(&ctx).await?;

    assert!(data.matched);

    Ok(())
}

#[test(tokio::test)]
async fn bool_type_conversion() -> Result<(), DataError> {
    let ctx = data_ctx("DUMMY_HASH");
    let rule = active_rule("SELECT 1 FROM call_trace_args cta WHERE as_boolean(cta.arg) = false;");

    let data = rule.verify(&ctx).await?;

    assert!(data.matched);

    Ok(())
}

#[test(tokio::test)]
async fn get_struct_field() -> Result<(), DataError> {
    let ctx = data_ctx("DUMMY_HASH");
    let rule = active_rule(
        "SELECT 1 FROM call_trace_args cta WHERE as_uint64(struct_field(cta.arg, 'foo')) = 42;",
    );

    let data = rule.verify(&ctx).await?;

    assert!(data.matched);

    Ok(())
}

#[test(tokio::test)]
async fn wrong_struct_field_does_not_fail() -> Result<(), DataError> {
    let ctx = data_ctx("DUMMY_HASH");
    let rule = active_rule(
        "SELECT 1 FROM call_trace_args cta WHERE as_uint64(struct_field(cta.arg, 'does-not-exist')) = 42;",
    );

    let data = rule.verify(&ctx).await?;

    assert!(!data.matched);

    Ok(())
}
