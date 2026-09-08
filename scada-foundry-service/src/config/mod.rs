use std::error::Error;

use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    config::{acse::AcseConfiguration, iccp::IccpConfiguration},
    error::{ScadaFoundryError, to_app_error},
};

pub mod acse;
pub mod iccp;

#[derive(Serialize, Deserialize)]
pub struct ApplicationConfiguration {
    #[serde(default = "default_resource")]
    pub filename: String,
    pub acse: AcseConfiguration,
    pub iccp: IccpConfiguration,
}

fn default_resource() -> String {
    String::from("")
}

impl ApplicationConfiguration {
    pub async fn new(filename: &str) -> Self {
        Self { filename: filename.into(), acse: AcseConfiguration { acse_listeners: vec![] }, iccp: IccpConfiguration { associations: vec![], data_points: vec![], transfer_sets: vec![] } }
    }

    pub async fn load(filename: &str) -> Result<ApplicationConfiguration, ScadaFoundryError> {
        let config_string: String = ApplicationConfiguration::_try_load_file(filename).await.map_err(to_app_error(format!("Failed to load application configuration: {filename}").as_str()))?;
        ApplicationConfiguration::_try_parse(config_string, filename.into()).await.map_err(to_app_error(format!("Failed to load application configuration: {filename}").as_str()))
    }

    pub async fn save(&self) -> Result<(), ScadaFoundryError> {
        self._try_save(self.filename.as_str()).await.map_err(to_app_error(format!("Failed to save application configuration: {}", self.filename).as_str())).into()
    }

    async fn _try_load_file(filename: &str) -> Result<String, std::io::Error> {
        let mut file = File::open(filename).await?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).await?;
        Ok(buffer)
    }

    async fn _try_parse(config_string: String, f: String) -> Result<ApplicationConfiguration, Box<dyn Error>> {
        let mut config = tokio::task::spawn_blocking(move || serde_json::from_str::<ApplicationConfiguration>(config_string.as_str())).await??;
        config.filename = f;
        Ok(config)
    }

    async fn _try_save(&self, filename: &str) -> Result<(), std::io::Error> {
        let json_data = serde_json::to_string_pretty(self)?;
        let mut file = File::create(filename).await?;
        file.write_all(&json_data.as_bytes()).await?;

        Ok(())
    }
}
