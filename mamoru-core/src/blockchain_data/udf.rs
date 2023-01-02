use crate::blockchain_data::value::Value;
use datafusion::arrow::array::{ArrayRef, BinaryArray, BooleanArray, StringArray, UInt64Array};
use datafusion::arrow::datatypes::DataType;
use datafusion::common::cast::{as_binary_array, as_string_array};
use datafusion::logical_expr::{create_udf, ScalarUDF, Volatility};
use datafusion::physical_plan::functions::make_scalar_function;
use std::sync::Arc;

macro_rules! define_as_ty_udf {
    ($f:ident, $ret:expr, $arr:ty, $code:tt) => {
        pub(crate) fn $f() -> ScalarUDF {
            let fun = make_scalar_function(|args: &[ArrayRef]| {
                let arg1 =
                    as_binary_array(&args[0]).expect("BUG: Failed to cast DataFusion value in UDF");

                let array = arg1
                    .iter()
                    .map(|data| match data {
                        Some(data) => $code(data),
                        _ => None,
                    })
                    .collect::<$arr>();

                Ok(Arc::new(array) as ArrayRef)
            });

            create_udf(
                stringify!($f),
                vec![DataType::Binary],
                Arc::new($ret),
                Volatility::Immutable,
                fun,
            )
        }
    };
}

define_as_ty_udf!(
    as_uint64,
    DataType::UInt64,
    UInt64Array,
    (|data| {
        let value = Value::from_slice(data).ok()?;

        value.as_u64()
    })
);

define_as_ty_udf!(
    as_boolean,
    DataType::Boolean,
    BooleanArray,
    (|data| {
        let value = Value::from_slice(data).ok()?;

        value.as_bool()
    })
);

define_as_ty_udf!(
    as_string,
    DataType::Utf8,
    StringArray,
    (|data| {
        let value = Value::from_slice(data).ok()?;

        value.into_string()
    })
);

pub(crate) fn struct_field() -> ScalarUDF {
    let fun = make_scalar_function(|args: &[ArrayRef]| {
        let arg1 = as_binary_array(&args[0]).expect("BUG: Failed to cast DataFusion value in UDF");
        let arg2 = as_string_array(&args[1]).expect("BUG: Failed to cast DataFusion value in UDF");

        let array = arg1
            .iter()
            .zip(arg2)
            .map(|(data, field)| match (data, field) {
                (Some(data), Some(field)) => {
                    let value = Value::from_slice(data).ok()?;
                    let field = &value.as_struct()?.fields.get(field)?;

                    field.serialize().ok()
                }
                _ => None,
            })
            .collect::<BinaryArray>();

        Ok(Arc::new(array) as ArrayRef)
    });

    create_udf(
        "struct_field",
        vec![DataType::Binary, DataType::Utf8],
        Arc::new(DataType::Binary),
        Volatility::Immutable,
        fun,
    )
}
