use crate::{
    connectors::{acse::AcseListenerOperationalReport, iccp::IccpServerAssociationOperationalReport}, iccp::api::IccpDataPointValue,
};

#[derive(Clone)]
pub enum ScadaFoundryEvent {
    AcseListenerReport(AcseListenerOperationalReport),

    // IccpAssociationUpdate(IccpServerOperationalAssociation),
    IccpDataPointUpdate(IccpDataPointValue),

    IccpServerAssociationReport(IccpServerAssociationOperationalReport),
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
