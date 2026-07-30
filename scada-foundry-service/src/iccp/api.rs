use std::collections::HashMap;

use async_trait::async_trait;
use rusty_iccp::IccpData;
use tokio::sync::mpsc::Sender;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum IccpDataPointName {
    Vcc(String),
    Icc(String, String),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct IccpDataPointKey {
    association_id: String,
    point_name: IccpDataPointName,
}

pub struct IccpDataPointValue {
    pub association_id: String,
    pub name: Option<String>,
    pub point_name: IccpDataPointKey,
    pub value: IccpData,

    pub source: String,         // The unique identifier of the source into this system
    pub recorded: u64,          // Recorded in this system
    pub updated: u64,           // Updated on this node
    pub timestamp: Option<u64>, // The timestamp received by SCADA

    pub allow_write: bool,
}

pub enum IccpAssociationState {
    Idle,
    Failed,
    Connected,
}

pub struct IccpAssociationStatus {
    id: String,
    name: String,
    enabled: bool,
    status_description: String,
    state: IccpAssociationState,
}

pub struct IccpAssociation {
    id: String,
    name: String,

    status: IccpAssociationStatus,
}

pub enum IccpSubsystemEvent {
    DataCenterState(IccpAssociationStatus),
    DataPointUpdate(IccpDataPointValue),
}
