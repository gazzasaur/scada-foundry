use std::{collections::HashMap, sync::Arc, time::SystemTime};

use futures::channel::mpsc::UnboundedSender;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DataPointsManager {
    listeners: Arc<Vec<UnboundedSender<DataPoint>>>,
    data_points: Arc<RwLock<HashMap<String, DataPoint>>>,
}

impl DataPointsManager {
    pub fn new() -> Self {
        DataPointsManager { data_points: Arc::new(RwLock::new(HashMap::new())), listeners: Arc::new(vec![]) }
    }

    pub async fn update_data_point(&self) {
        let data_points = self.data_points.clone();
    }

    pub async fn serve(&self) {
        let data_points = self.data_points.clone();
    }
}

#[derive(Clone)]
pub struct DataPoint {
    pub uuid: String,
    pub name: String,

    pub value: f64,
    pub override_value: f64,
    pub flags: Vec<DataPointFlags>,

    pub valid: bool,
    pub overidden: bool,
    pub local_timestamp: bool,

    pub value_updated: SystemTime,
    pub value_received: SystemTime,
    pub status_updated: SystemTime,

    pub last_updated_source: String,
}

#[derive(Clone)]
pub enum DataPointFlags {
    Measured,
    Calculated,
    CommunicationsLost,
}
