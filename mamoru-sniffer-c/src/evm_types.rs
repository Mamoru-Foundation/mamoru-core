use crate::ffi_types::FfiBlockchainData;
use mamoru_evm_types::{Transaction, TransactionBatch};
use safer_ffi::prelude::*;

#[derive_ReprC]
#[ReprC::opaque]
pub struct FfiTransactionBatch {
    pub inner: Box<TransactionBatch>,
}

#[ffi_export]
fn new_transaction_batch() -> repr_c::Box<FfiTransactionBatch> {
    let inner = Box::new(TransactionBatch(Vec::new()));

    repr_c::Box::new(FfiTransactionBatch { inner })
}

#[ffi_export]
fn transaction_batch_append<'a>(
    batch: &mut FfiTransactionBatch,
    index: u64,
    block_index: u64,
    hash: c_slice::Ref<'a, u8>,
    typ: u8,
    nonce: u32,
    status: char_p::Ref<'a>,
    timestamp: u32,
    from: c_slice::Ref<'a, u8>,
    to: c_slice::Ref<'a, u8>,
) {
    let batch = &mut batch.inner.0;

    batch.push(Transaction {
        index,
        block_index,
        hash: hash.to_vec(),
        typ,
        nonce,
        status: status.to_str().to_string(),
        timestamp,
        from: from.to_vec(),
        to: to.to_vec(),
    });
}

/// Frees `object` argument.
#[ffi_export]
fn transaction_batch_finish(
    object: repr_c::Box<FfiTransactionBatch>,
) -> repr_c::Box<FfiBlockchainData> {
    let inner = object.into().inner;

    repr_c::Box::new(FfiBlockchainData { inner })
}
