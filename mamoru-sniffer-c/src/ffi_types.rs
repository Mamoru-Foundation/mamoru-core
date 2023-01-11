use mamoru_sniffer::core::{
    BlockchainData, BlockchainDataCtx, BlockchainDataCtxBuilder, Value, ValueData,
};
use mamoru_sniffer::{Sniffer, SnifferError};
use safer_ffi::prelude::*;

#[derive_ReprC]
#[ReprC::opaque]
pub struct FfiSniffer {
    pub(crate) inner: Sniffer,
}

#[derive_ReprC]
#[ReprC::opaque]
pub struct FfiSnifferResult<T> {
    pub(crate) data: Option<T>,
    pub(crate) error: Option<SnifferError>,
}

#[derive_ReprC]
#[ReprC::opaque]
pub struct FfiBlockchainData {
    pub inner: Box<dyn BlockchainData>,
}

#[derive_ReprC]
#[ReprC::opaque]
pub struct FfiBlockchainDataCtx {
    pub(crate) inner: BlockchainDataCtx,
}

#[derive_ReprC]
#[ReprC::opaque]
pub struct FfiBlockchainDataCtxBuilder {
    pub(crate) inner: BlockchainDataCtxBuilder,
}

#[derive_ReprC]
#[ReprC::opaque]
pub struct FfiValueData {
    pub inner: ValueData,
}

#[derive_ReprC]
#[ReprC::opaque]
pub struct FfiValue {
    pub(crate) inner: Value,
}
