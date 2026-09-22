use crate::{
    connectors::{acse::AcseListenerOperationalReport, iccp::IccpServerAssocaitionOperationalReport}, iccp::api::IccpDataPointValue,
};

#[derive(Clone)]
pub enum ScadaFoundryEvent {
    AcseListenerReport(AcseListenerOperationalReport),

    // IccpAssociationUpdate(IccpServerOperationalAssociation),
    IccpDataPointUpdate(IccpDataPointValue),

    IccpServerAssociationReport(IccpServerAssocaitionOperationalReport),
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
