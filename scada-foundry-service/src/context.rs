use std::collections::{
    HashMap,
    hash_map::Entry::{Occupied, Vacant},
};

use anyhow::anyhow;
use rusty_iccp::IccpData::{self, RealQ};
use tokio::sync::mpsc::Sender;

pub struct IccpSubsystem {
    data_centers: HashMap<String, IccpDataCenter>,
    data_points: HashMap<IccpDataPointKey, IccpDataPointValue>,

    listener: Sender<IccpSubsystemEvent>,
}
impl IccpSubsystem {
    pub fn update_data_point(&mut self, control_center: String, data_point_key: IccpDataPointKey, value: IccpData) -> Result<(), anyhow::Error> {
        let data_point = match self.data_points.get_mut(&data_point_key) {
            Some(x) => x,
            None => todo!(),
        };
        match value {
            IccpData::RealQ(_, _, _, _, _) => match data_point.value {
                IccpData::RealQ(_, _, _, _, _) => data_point.value = value,
                _ => return Err(anyhow!("incompatible types")),
            },
            IccpData::State(_, _, _, _, _) => match data_point.value {
                IccpData::State(_, _, _, _, _) => data_point.value = value,
                _ => return Err(anyhow!("incompatible types")),
            },
            IccpData::DiscreteQ(_, _, _, _, _) => match data_point.value {
                IccpData::DiscreteQ(_, _, _, _, _) => data_point.value = value,
                _ => return Err(anyhow!("incompatible types")),
            },
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ApplicationContext {}
