use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AcseListener {
    OsiStackAcseListener {
        id: String,
        host: String,
        port: u16,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcseConfiguration {
    pub acse_listeners: Vec<AcseListener>,
}
