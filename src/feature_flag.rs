use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct FeatureFlagPayload {
    pub key: String,
    pub distinct_id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub groups: HashMap<String, String>,
}

impl FeatureFlagPayload {
    /// Crate a new feature flag payload
    pub fn new<S: Into<String>>(key: S, distinct_id: S) -> Self {
        Self {
            key: key.into(),
            distinct_id: distinct_id.into(),
            properties: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    /// Add a property to the event
    ///
    /// Errors if `prop` fails to serialize
    pub fn insert_prop<K: Into<String>, P: Serialize>(
        &mut self,
        key: K,
        prop: P,
    ) -> Result<(), Error> {
        let as_json =
            serde_json::to_value(prop).map_err(|e| Error::Serialization(e.to_string()))?;
        let _ = self.properties.insert(key.into(), as_json);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DecideRequestData {
    pub api_key: String,
    pub distinct_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DecideResponse {
    #[serde(rename = "featureFlags")]
    pub feature_flags: HashMap<String, bool>,
    #[serde(rename = "featureFlagPayloads")]
    pub feature_flag_payloads: HashMap<String, String>,
}
