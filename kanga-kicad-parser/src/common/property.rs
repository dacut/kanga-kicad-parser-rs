use serde::{Deserialize, Serialize};

/// KiCad key-value property.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "property")]
pub struct Property {
    /// Property name.
    pub key: String,

    /// Property value.
    pub value: String,
}
