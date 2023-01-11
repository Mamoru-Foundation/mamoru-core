mod errors;
mod sniffer;

pub mod validation_chain;
pub use errors::*;
pub use sniffer::*;
pub mod core {
    pub use mamoru_core::*;
}

fn from_env<T>() -> Result<T, envy::Error>
where
    T: serde::de::DeserializeOwned,
{
    envy::prefixed("MAMORU_").from_env()
}

fn from_env_or_fail<T>() -> T
where
    T: serde::de::DeserializeOwned,
{
    from_env().expect("Missing environment variables")
}
