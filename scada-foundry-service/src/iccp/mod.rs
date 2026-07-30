use std::collections::{HashMap, hash_map::Entry::{Occupied, Vacant}};

use crate::{config::iccp::IccpAssociation, error::ScadaFoundryError};

pub mod api;
pub mod converter;

pub struct IccpSubsystem {
    associations: HashMap<String, IccpAssociation>
    // fn set_listener(listener: Sender<IccpSubsystemEvent>);

    // async fn create_association(association: IccpAssociation);

    // async fn fetch_data_points() -> Vec<IccpDataPointValue>;

    // fn create_data_point(data_point: IccpDataPointValue);
    // fn update_data_point(data_point: IccpDataPointValue);
    // fn delete_data_point(data_point_key: IccpDataPointKey);
    // fn update_data_point_value(data_point_key: IccpDataPointKey, source: String, iccp_data_point_value: IccpDataPointValue);
}

impl IccpSubsystem {
    pub async fn new() -> Self {
        Self { associations: HashMap::new() }
    }

    pub async fn list_associations(&self) -> Vec<String> {
        return self.associations.values().map(|assoc| assoc.id.clone()).collect();
    }

    pub async fn create_association(&mut self, association: IccpAssociation) -> Result<(), ScadaFoundryError> {
        match self.associations.entry(association.id.clone()) {
            Occupied(_) => return Err(ScadaFoundryError::ApplicationError("association already exists".into())),
            Vacant(vacant_entry) => vacant_entry.insert(association),
        };
        Ok(())
    }
}

async fn process_association_connect(mut association: IccpAssociation) {

}