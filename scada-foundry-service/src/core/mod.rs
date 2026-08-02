use std::{collections::HashMap, sync::Arc, time::SystemTime};

use futures::channel::mpsc::UnboundedSender;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DataPointsManager {
    internal_manager: Arc<RwLock<DataPointsManagerInternal>>,
}

impl DataPointsManager {
    pub fn new() -> Self {
        DataPointsManager { internal_manager: Arc::new(RwLock::new(DataPointsManagerInternal::new())) }
    }

    pub async fn update_data_point(&self) {
        // let data_points: Arc<RwLock<HashMap<String, DataPoint>>> = self.data_points.clone();
    }

    pub async fn serve(&self) {
        // let data_points = self.data_points.clone();
    }
}

#[derive(Clone)]
pub struct DataPointsManagerInternal {
    data_points: HashMap<String, DataPoint>,
    listeners: Vec<UnboundedSender<DataPoint>>,
}

impl DataPointsManagerInternal {
    pub fn new() -> Self {
        Self { data_points: HashMap::new(), listeners: Vec::new() }
    }

    pub async fn update_data_point_value(&self, uuid: &String, value: f64) {
        // let data_points: Arc<RwLock<HashMap<String, DataPoint>>> = self.data_points.clone();
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

    // Event Timestamp Source (Local)
    // Locally Updated Timestamp
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
