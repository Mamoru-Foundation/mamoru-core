mod evm_types;
mod ffi_types;

use lazy_static::lazy_static;
use mamoru_sniffer::{
    core::{BlockchainDataCtxBuilder, StructValue, Value, ValueData},
    Sniffer, SnifferConfig,
};
use safer_ffi::prelude::*;
use std::collections::HashMap;

pub use evm_types::*;
pub use ffi_types::*;

lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("BUG: Failed to init async runtime.")
    };
}

#[ffi_export]
fn new_sniffer() -> repr_c::Box<FfiSnifferResult<FfiSniffer>> {
    let result = RUNTIME.block_on(async {
        let config = SnifferConfig::from_env()?;

        Sniffer::new(config).await
    });

    let (data, error) = match result {
        Ok(sniffer) => (Some(FfiSniffer { inner: sniffer }), None),
        Err(err) => (None, Some(err)),
    };

    repr_c::Box::new(FfiSnifferResult { data, error })
}

/// Returns a pointer to C-like string or NULL if there is no error.
#[ffi_export]
fn sniffer_result_get_error_message(result: &FfiSnifferResult<FfiSniffer>) -> Option<char_p::Box> {
    result
        .error
        .as_ref()
        .map(|err| format!("{:#?}", err).try_into().unwrap())
}

/// Returns a pointer to `FfiSniffer` or NULL.
/// Frees `result` arguments, so be sure to call `sniffer_result_get_error_message` to get an error message.
#[ffi_export]
fn sniffer_result_get_sniffer(
    result: repr_c::Box<FfiSnifferResult<FfiSniffer>>,
) -> Option<repr_c::Box<FfiSniffer>> {
    let result = result.into();

    result.data.map(|sniffer| repr_c::Box::new(sniffer))
}

/// Frees `data` argument.
#[ffi_export]
fn sniffer_observe_data(sniffer: &FfiSniffer, data: repr_c::Box<FfiBlockchainDataCtx>) {
    let sniffer = &sniffer.inner;
    let data = data.into();

    RUNTIME.block_on(async { sniffer.observe_data(data.inner).await });
}

#[ffi_export]
fn new_blockchain_data_ctx_builder() -> repr_c::Box<FfiBlockchainDataCtxBuilder> {
    let inner = BlockchainDataCtxBuilder::new();

    repr_c::Box::new(FfiBlockchainDataCtxBuilder { inner })
}

/// Returns `true` is success.
/// Frees `data` argument.
#[ffi_export]
fn blockchain_data_ctx_builder_add_data(
    builder: &FfiBlockchainDataCtxBuilder,
    data: repr_c::Box<FfiBlockchainData>,
) -> bool {
    let builder = &builder.inner;
    let data = data.into().inner;

    builder.add_data(data).is_ok()
}

#[ffi_export]
fn blockchain_data_ctx_builder_finish(
    builder: repr_c::Box<FfiBlockchainDataCtxBuilder>,
    tx_id: char_p::Ref<'_>,
    tx_hash: char_p::Ref<'_>,
    tx_timestamp_secs: i64,
) -> repr_c::Box<FfiBlockchainDataCtx> {
    let builder = builder.into().inner;
    let tx_id = tx_id.to_str().to_string();
    let tx_hash = tx_hash.to_str().to_string();

    let now = chrono::NaiveDateTime::from_timestamp_opt(tx_timestamp_secs, 0).unwrap_or_default();

    repr_c::Box::new(FfiBlockchainDataCtx {
        inner: builder.finish(tx_id, tx_hash, now),
    })
}

/// Frees `value` argument.
#[ffi_export]
fn new_value_data(value: repr_c::Box<FfiValue>) -> repr_c::Box<FfiValueData> {
    let value = value.into().inner;

    repr_c::Box::new(FfiValueData {
        inner: ValueData::new(value).expect("BUG: `Value` serialization failed."),
    })
}

#[ffi_export]
fn new_value_bool(data: bool) -> repr_c::Box<FfiValue> {
    repr_c::Box::new(FfiValue {
        inner: Value::Bool(data),
    })
}

#[ffi_export]
fn new_value_u64(data: u64) -> repr_c::Box<FfiValue> {
    repr_c::Box::new(FfiValue {
        inner: Value::U64(data),
    })
}

#[ffi_export]
fn new_value_list() -> repr_c::Box<FfiValue> {
    repr_c::Box::new(FfiValue {
        inner: Value::List(vec![]),
    })
}

/// Returns `true` if success.
/// Frees `data` argument.
#[ffi_export]
fn value_list_append(value_list: &mut FfiValue, data: repr_c::Box<FfiValue>) -> bool {
    let mut ok = false;
    let data = data.into().inner;

    if let Value::List(list) = &mut value_list.inner {
        list.push(data);

        ok = true;
    }

    ok
}

#[ffi_export]
fn new_value_struct(ty: char_p::Ref<'_>) -> repr_c::Box<FfiValue> {
    let ty = ty.to_str().to_string();

    repr_c::Box::new(FfiValue {
        inner: Value::Struct(StructValue::new(ty, HashMap::new())),
    })
}

/// Returns `true` if success.
/// Frees `data` argument.
#[ffi_export]
fn value_struct_add_field(
    value_struct: &mut FfiValue,
    key: char_p::Ref<'_>,
    data: repr_c::Box<FfiValue>,
) -> bool {
    let data = data.into().inner;
    let key = key.to_str().to_string();

    if let Value::Struct(value) = &mut value_struct.inner {
        if value.field_exist(&key) {
            return false;
        } else {
            value.add_field(key, data);
        }
    } else {
        return false;
    }

    true
}

#[safer_ffi::cfg_headers]
#[test]
fn generate_headers() -> ::std::io::Result<()> {
    safer_ffi::headers::builder()
        .to_file("headers.h")?
        .generate()
}
