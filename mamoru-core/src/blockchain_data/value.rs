use crate::ValueError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::error;

/// Holds serialized [`Value`]
pub struct ValueData {
    data: Vec<u8>,
}

impl ValueData {
    pub fn new(value: Value) -> Option<Self> {
        Some(Self {
            data: value.serialize().ok()?,
        })
    }
}

impl AsRef<[u8]> for ValueData {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

/// Represents variable types passed from blockchains.
/// The end user can work with it via UDFs.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    Bool(bool),
    U64(u64),
    String(String),
    List(Vec<Value>),
    Struct(StructValue),
}

impl Value {
    pub(crate) fn serialize(&self) -> Result<Vec<u8>, ValueError> {
        bincode::serialize(self).map_err(|err| {
            error!(error = ?err, value = ?self, "Failed to serialize `Value`.");

            ValueError::Serialize(Box::new(err))
        })
    }

    pub(crate) fn from_slice(data: &[u8]) -> Result<Self, ValueError> {
        bincode::deserialize(data).map_err(|err| ValueError::Deserialize(Box::new(err)))
    }
}

impl Value {
    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn as_u64(&self) -> Option<u64> {
        match self {
            Value::U64(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn into_string(self) -> Option<String> {
        match self {
            Value::String(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_struct(&self) -> Option<&StructValue> {
        match self {
            Value::Struct(value) => Some(value),
            _ => None,
        }
    }
}

/// Represents Struct types passed from blockchains.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructValue {
    pub(crate) ty: String,
    pub(crate) fields: HashMap<String, Value>,
}

impl StructValue {
    pub fn new(ty: String, fields: HashMap<String, Value>) -> Self {
        Self { ty, fields }
    }

    pub fn add_field(&mut self, key: String, value: Value) -> Option<Value> {
        self.fields.insert(key, value)
    }

    pub fn field_exist(&self, key: &str) -> bool {
        self.fields.get(key).is_some()
    }
}
