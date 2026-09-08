use crate::{connectors::acse::AcseConnectorStatus, iccp::api::{IccpDataPointValue, IccpServerOperationalAssociation}};

#[derive(Clone)]
pub enum ScadaFoundryEvent {
    AcseConnectorStatusUpdate(AcseConnectorStatus),

    IccpAssociationUpdate(IccpServerOperationalAssociation),
    IccpDataPointUpdate(IccpDataPointValue),
}

pub struct AcseControlCentre {
    pub tsap: Vec<u8>,
    pub ssap: Vec<u8>,
    pub psap: Vec<u8>,
}

pub struct AcseProtocolInformation {
    pub called: AcseControlCentre,
    pub calling: AcseControlCentre,
}

