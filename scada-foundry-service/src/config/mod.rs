use serde::{Deserialize, Serialize};

use crate::{config::iccp::IccpConfiguration, error::{ScadaFoundryError, to_app_error}};

pub mod iccp;

#[derive(Serialize, Deserialize)]
pub struct ApplicationConfiguration {
    iccp: IccpConfiguration
}

impl ApplicationConfiguration {
    async fn save(&self, filename: &str) -> Result<(), ScadaFoundryError> {
        let json_data = serde_json::to_vec_pretty(self).map_err(to_app_error(format!("Failed to save application configuration: {filename}").as_str()))?;
    // let mut file = File::create("config.json").await?;
    // file.write_all(&json_data).await?;
        Ok(())
    }
}