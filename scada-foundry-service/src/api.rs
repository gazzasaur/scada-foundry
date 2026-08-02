use crate::iccp::api::{IccpAssociationStatus, IccpDataPointValue};

pub enum ScadaFoundryEvent {
    IccpAssociationUpdate(IccpAssociationStatus),
    IccpDataPointUpdate(IccpDataPointValue),
}
