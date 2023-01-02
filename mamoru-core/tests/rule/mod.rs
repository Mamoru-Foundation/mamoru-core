use mamoru_core::Rule;

mod smoke;
mod udf;

pub fn active_rule(expression: impl AsRef<str>) -> Rule {
    Rule::new("dummy".to_string(), 0, i64::MAX, expression.as_ref())
        .expect("Failed to create rule.")
}

pub fn inactive_rule(expression: impl AsRef<str>) -> Rule {
    Rule::new("dummy".to_string(), 0, 0, expression.as_ref()).expect("Failed to create rule.")
}
