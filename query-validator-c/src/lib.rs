use lazy_static::lazy_static;
use query_validator::{validate, ChainType};
use safer_ffi::prelude::*;

#[derive_ReprC]
#[repr(u8)]
pub enum FfiChainType {
    Sui = 0,
    Evm = 1,
}

#[derive_ReprC]
#[repr(C)]
pub struct FfiValidationResult {
    is_error: bool,
    message: char_p::Box,
}

impl From<FfiChainType> for ChainType {
    fn from(chain: FfiChainType) -> Self {
        match chain {
            FfiChainType::Sui => Self::Sui,
            FfiChainType::Evm => Self::Evm,
        }
    }
}

lazy_static! {
    static ref HANDLE: tokio::runtime::Handle = {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("BUG: Failed to init async runtime.");

        runtime.handle().clone()
    };
}

#[ffi_export]
fn ffi_validate(chain: FfiChainType, query: char_p::Ref<'_>) -> FfiValidationResult {
    let chain = chain.into();
    let query = query.to_str();

    let result = HANDLE.block_on(async move { validate(chain, query).await });

    let (is_error, message) = match result {
        Ok(_) => (false, "".to_string()),
        Err(err) => (true, format!("{:#?}", err)),
    };

    FfiValidationResult {
        is_error,
        message: message.try_into().unwrap(),
    }
}

#[ffi_export]
fn ffi_drop_validation_result(result: FfiValidationResult) {
    drop(result)
}

#[safer_ffi::cfg_headers]
#[test]
fn generate_headers() -> ::std::io::Result<()> {
    safer_ffi::headers::builder()
        .to_file("headers.h")?
        .generate()
}
