use crate::iccp::api::{IccpAssociationState, IccpDataPointValue};

#[derive(Clone, Debug)]
pub enum ScadaFoundryEvent {
    IccpAssociationUpdate(IccpAssociationState),
    IccpDataPointUpdate(IccpDataPointValue),
}
