use std::collections::HashMap;

use async_trait::async_trait;
use rusty_iccp::IccpData;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use crate::config::iccp::IccpDataPoint;

pub struct IccpDataPointPermission {
    principal_id: String,
    allow_read: bool,
    allow_write: bool,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct IccpDataPointKey {
    partition: String,
    domain: Option<String>,
    point_name: String,
}

pub struct IccpDataPointValue {
    pub partition: String,
    pub domain: Option<String>,
    pub point_name: String,
    pub value: IccpData,

    pub source: String,         // The unique identifier of the source into this system
    pub recorded: u64,          // Recorded in this system
    pub updated: u64,           // Updated on this node
    pub timestamp: Option<u64>, // The timestamp received by SCADA

    pub default_allow_read: bool,
    pub default_allow_write: bool,
    pub permissions: HashMap<String, IccpDataPointPermission>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IccpDeviceKey {
    partition: String,
    domain: Option<String>,
    device_name: String,
}

pub enum IccpDataCenterState {
    Idle,
    Failed,
    Connected,
}

pub enum IccpDataCenterControl {
    Enabled,
    Disabled,
}

pub struct IccpDataCenterStatus {
    id: String,
    name: String,
    status_description: String,
    state: IccpDataCenterState,
}

pub struct IccpDataCenter {
    id: String,
    name: String,
    partition: String,

    status: IccpDataCenterStatus,
    control: IccpDataCenterControl,
}

pub enum IccpSubsystemEvent {
    DataCenterState(IccpDataCenterStatus),
    DataPointUpdate(IccpDataPointValue),
}

#[async_trait]
pub trait IccpSubsystem {
    fn set_listener(listener: Sender<IccpSubsystemEvent>);

    async fn register_association();

    async fn list_data_points() -> Vec<IccpDataPoint>;
    async fn fetch_data_point(source: String, data_point_key: IccpDataPointKey);

    fn create_data_point(data_point: IccpDataPoint);
    fn update_data_point(data_point: IccpDataPoint);
    fn delete_data_point(data_point_key: IccpDataPointKey);
    fn update_data_point_value(data_point_key: IccpDataPointKey, source: String, iccp_data_point_value: IccpDataPointValue);
}