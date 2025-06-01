use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use egui_snarl::Snarl;

use crate::node::{OSRCNode, PinType};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FPGAProgram {
    pub info: FileInfo,
    pub set_node_vars: Vec<GlobalVariable>,
    pub networks: Vec<Network>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FileInfo {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub fpga_config: String,
    pub write_protected: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GlobalVariable {
    pub name: String,
    pub pin_type: PinType,
}

impl GlobalVariable {
    pub fn new() -> Self {
        Self {
            name: "New Variable".to_string(),
            pin_type: PinType::UNDEFINED,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Network {
    pub name: String,
    pub enabled: bool,
    pub net_type: NetworkType,
    pub timeout: u32,
    pub update_cycle_trigger_count: u32,
    pub execution_index: u32,
    pub nodes: Snarl<OSRCNode>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum NetworkType {
    Sync,
    Async,
}

impl Network {
    pub fn new() -> Self {
        let name: String = "New Network".to_string();
        Network {
            name,
            enabled: true,
            net_type: NetworkType::Sync,
            timeout: 10000,
            update_cycle_trigger_count: 1,
            execution_index: 0,
            nodes: Snarl::new(),
        }
    }
}
