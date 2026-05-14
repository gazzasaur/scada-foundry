use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::iccp::IccpConfiguration;

pub struct IccpManager {
    config: Arc<RwLock<Option<IccpConfiguration>>>,
}

impl IccpManager {
    pub fn new() -> Self {
        IccpManager { config: Arc::new(RwLock::new(None)) }
    }

    pub async fn apply_config(config: IccpConfiguration) {
        config.
    }
}

struct IccpManagerWorker {
    config: Arc<RwLock<Option<IccpConfiguration>>>,
}
