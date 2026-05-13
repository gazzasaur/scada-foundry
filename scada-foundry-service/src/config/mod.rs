use serde::{Deserialize, Serialize};

use crate::config::iccp::IccpConfiguration;

pub mod iccp;

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    iccp: IccpConfiguration
}
