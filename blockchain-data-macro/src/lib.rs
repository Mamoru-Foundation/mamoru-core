pub use blockchain_data_derive::BlockchainData;

pub mod vendor {
    pub use paste::paste;
}

#[macro_export]
macro_rules! tables {
    ($($t:ty,)+) => {
        blockchain_data_macro::vendor::paste!{ Ok(vec![
            $(
                ([< $t Batch >]::new(vec![]).table_name(), [< $t Batch >]::new(vec![]).boxed().to_record_batch()?),
            )+
        ]) }
    }
}
