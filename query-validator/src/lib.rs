mod error;
pub use error::*;

use mamoru_core::{BlockchainDataCtxBuilder, DataError, Rule};

/// Represents possible blockchains as each one has different schema
#[derive(Debug)]
pub enum ChainType {
    Sui,
    Evm,
}

/// Validates a Rule expression against an empty database.
pub async fn validate(chain: ChainType, query: &str) -> Result<(), ValidateError> {
    let ctx = match chain {
        ChainType::Sui => BlockchainDataCtxBuilder::new()
            .empty(mamoru_sui_types::all_tables)
            .expect(
                "BUG: `BlockchainDataCtxBuilder::new().empty(mamoru_sui_types::all_tables)` fails.",
            ),

        ChainType::Evm => BlockchainDataCtxBuilder::new()
            .empty(mamoru_evm_types::all_tables)
            .expect(
                "BUG: `BlockchainDataCtxBuilder::new().empty(mamoru_evm_types::all_tables)` fails.",
            ),
    };

    let rule = validation_rule(query)?;
    let result = rule.verify(&ctx).await?;

    if result.matched {
        return Err(ValidateError::MatchesEmptyDatabase);
    }

    Ok(())
}

fn validation_rule(query: &str) -> Result<Rule, DataError> {
    Rule::new("QUERY_VALIDATE".to_string(), 0, i64::MAX, query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sui_empty_ctx_does_not_fail() {
        BlockchainDataCtxBuilder::new()
            .empty(mamoru_sui_types::all_tables)
            .unwrap();
    }

    #[tokio::test]
    async fn valid_expression_ok() {
        let result = validate(ChainType::Sui, "SELECT * FROM transactions").await;

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn always_true_expression_fails() {
        let result = validate(ChainType::Sui, "SELECT 1").await;

        assert!(matches!(result, Err(ValidateError::MatchesEmptyDatabase)))
    }

    #[tokio::test]
    async fn wrong_table_name_fails() {
        let result = validate(ChainType::Sui, "SELECT * FROM THIS_TABLE_DOES_NOT_EXIST").await;

        assert!(matches!(
            result,
            Err(ValidateError::DataError(DataError::PlanQuery(_)))
        ))
    }
}
