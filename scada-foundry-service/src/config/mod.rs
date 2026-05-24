use std::error::Error;

use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    config::iccp::IccpConfiguration,
    error::{ScadaFoundryError, to_app_error},
};

pub mod iccp;

#[derive(Serialize, Deserialize)]
pub struct ApplicationConfiguration {
    pub iccp: IccpConfiguration,
}

impl ApplicationConfiguration {
    pub async fn load(filename: &str) -> Result<ApplicationConfiguration, ScadaFoundryError> {
        let config_string: String = ApplicationConfiguration::_try_load_file(filename).await.map_err(to_app_error(format!("Failed to load application configuration: {filename}").as_str()))?;
        ApplicationConfiguration::_try_parse(config_string).await.map_err(to_app_error(format!("Failed to load application configuration: {filename}").as_str()))
    }

    pub async fn save(&self, filename: &str) -> Result<(), ScadaFoundryError> {
        self._try_save(filename).await.map_err(to_app_error(format!("Failed to save application configuration: {filename}").as_str())).into()
    }

    async fn _try_load_file(filename: &str) -> Result<String, std::io::Error> {
        let mut file = File::create(filename).await?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).await?;
        Ok(buffer)
    }

    async fn _try_parse(config_string: String) -> Result<ApplicationConfiguration, Box<dyn Error>> {
        Ok(tokio::task::spawn_blocking(move || serde_json::from_str::<ApplicationConfiguration>(config_string.as_str())).await??)
    }

    async fn _try_save(&self, filename: &str) -> Result<(), std::io::Error> {
        let json_data = serde_json::to_vec_pretty(self)?;
        let mut file = File::create(filename).await?;
        file.write_all(&json_data).await?;

        Ok(())
    }
}
