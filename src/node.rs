use std::fmt::{Debug, Display};

use convert_case::Casing;
use egui::{Color32, ComboBox, Ui};
use egui_snarl::{InPin, NodeId, OutPin, Snarl, ui::PinInfo};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr, VariantArray};

use derive_alias::derive_alias;
use serde::Deserialize;
use serde::Serialize;

derive_alias! {
    derive_node => #[derive(Clone, Serialize, Deserialize, IntoStaticStr, EnumIter, Debug, Display, Default, PartialEq)]
}

derive_node! {
pub enum OSRCNode {
    Constant {
        itype: PinType,
        value: String,
        node_name: String,
    },
    // Api calls would set this value
    ApiInput {
        min: Option<f64>,
        max: Option<f64>,
        default: Option<f64>,
        timeout: Option<f64>,
        itype: PinType,
        node_name: String,
    },
    // The api would read this value
    ApiOutput {
        itype: PinType,
        node_name: String,
    },
    PIController {
        p: f32,
        i: f32,
        i_limit: f32,
        output_min: f32,
        output_max: f32,
        node_name: String,
    },
    VelEstimator {
        alpha: f32,
    },
    LogicGate {
        gtype: GateType,
    },
    Comparator {
        itype: PinType,
        comparison: ValueCompare
    },
    MathOperation {
        itype: PinType,
        operator: MathOperation,
    },
    Multiplexer {
        itype: PinType,
        input_bits: usize
    },
    BitwiseSplit {
        num_bits: usize,
    },
    BitwiseJoin {
        num_bits: usize,
    },
    EdgeDelay {
        cycles: usize,
        rising_edge: bool,
        falling_edge: bool,
    },
    CycleDelay {
        cycles: usize,
        itype: PinType,
    },
    Converter {
        input_type: PinType,
        output_type: PinType,
        direct_mode: bool,
        input_min: String,
        input_max: String,
        output_min: String,
        output_max: String,
        invert: bool,
    },
    SerialDevice {
        enabled: bool,
        addr: u16,
        timeout: u16,
        descriptor: String,
        num_read: usize,
        num_write: usize,
        node_name: String,
    },
    SerialRead {
        name: String,
        dev: SerialDeviceReg,
        itype: PinType,
    },
    SerialWrite {
        name: String,
        dev: SerialDeviceReg,
        itype: PinType,
    },
    GlobalVariableInput {
        name: String,
    },
    GlobalVariableOutput {
        name: String,
    },
    #[strum(disabled)]
    #[default]
    Invalid
}
}

derive_node! {
    pub enum ValueCompare {
        GreaterThan,
        GreaterThanOrEqual,
        LessThan,
        LessThanOrEqual,
        #[default]
        Equal,
        NotEqual,
    }
}

derive_node! {
pub enum GateType {
    AND(usize),
    OR(usize),
    NAND(usize),
    NOR(usize),
    XOR(usize),
    #[default]
    NOT
}
}

derive_node! {
pub enum MathOperation {
    Nary(NaryOperation, usize),
    BinaryOperation(BinaryOperation),
    UnaryOperation(UnaryOperation),
    #[default]
    Invalid
}
}

derive_node! {
pub enum NaryOperation {
    #[default]
    ADD,
    MUL,
    MIN,
    MAX,
    AVG
}
}

derive_node! {
pub enum BinaryOperation {
    #[default]
    ROOT,
    POW,
    LOG,
}
}

derive_node! {
pub enum UnaryOperation {
    #[default]
    LN,
    LOG10,
    SINE,
    COS,
    TAN,
    ASIN,
    ACOS,
    ATAN,
    NEGATE,
    ABS,
}
}

impl OSRCNode {
    pub fn name(&self) -> String {
        let s: &str = self.into();
        s.to_string().to_case(convert_case::Case::Title)
    }

    pub fn pin_type_input(&self, idx: usize) -> PinType {
        match (self, idx) {
            (OSRCNode::ApiOutput { itype, .. }, idx) => *itype,
            (OSRCNode::BitwiseSplit { num_bits }, _) => PinType::from_size(*num_bits),
            (OSRCNode::BitwiseJoin { num_bits }, _) => PinType::Bool,
            (OSRCNode::EdgeDelay { cycles, .. }, _) => PinType::Bool,
            (OSRCNode::CycleDelay { cycles, itype, .. }, _) => *itype,
            (OSRCNode::Converter { input_type,   .. }, _) => *input_type,
            (OSRCNode::LogicGate { .. }, _) => PinType::Bool,
            (OSRCNode::SerialDevice { .. }, idx) => OSRCNode::serial_type(idx),
            (OSRCNode::SerialWrite { itype, .. }, _) => *itype,
            (OSRCNode::SerialRead { .. }, _) => PinType::SERIAL,
            (OSRCNode::GlobalVariableOutput { .. }, _) => PinType::NONE,
            (OSRCNode::GlobalVariableInput { .. }, _) => PinType::UNDEFINED,
            (OSRCNode::MathOperation { itype, .. }, _) => *itype,
            (OSRCNode::ApiInput { .. }, _) => PinType::NONE,
            (OSRCNode::Invalid, _) => PinType::NONE,
            (OSRCNode::Constant { .. }, _) => PinType::NONE,
            (OSRCNode::Comparator { itype, .. }, idx) => *itype,
            (OSRCNode::Multiplexer { itype, input_bits }, idx) => {
                if idx < *input_bits {
                    PinType::Bool
                } else {
                    *itype
                }
            },
            (OSRCNode::PIController { .. }, _) => PinType::F32,
            (OSRCNode::VelEstimator { .. }, _) => PinType::F64,
        }
    }

    fn serial_type(i: usize) -> PinType {
        match i {
            0 => PinType::Bool,
            _ => PinType::SERIAL,
        }
    }

    pub fn pin_type_output(&self, idx: usize) -> PinType {
        match (self, idx) {
            (OSRCNode::ApiInput { itype, .. }, idx) => *itype,
            // (OSRCNode::AndGate { inputs: _ }, _) => PinType::Bool,
            // (OSRCNode::OrGate { inputs: _ }, _) => PinType::Bool,
            // (OSRCNode::NotGate, _) => PinType::Bool,
            // (OSRCNode::NandGate { inputs: _ }, _) => PinType::Bool,
            // (OSRCNode::NorGate { inputs: _ }, _) => PinType::Bool,
            (OSRCNode::BitwiseSplit { num_bits }, _) => PinType::Bool,
            (OSRCNode::BitwiseJoin { num_bits }, _) => PinType::from_size(*num_bits),
            (OSRCNode::EdgeDelay { cycles, .. }, _) => PinType::Bool,
            (OSRCNode::CycleDelay { cycles, itype, .. }, _) => *itype,
            (OSRCNode::Converter { output_type, .. }, _) => *output_type,
            (OSRCNode::LogicGate { gtype }, _) => PinType::Bool,
            (OSRCNode::SerialDevice { .. }, _) => PinType::SERIAL,
            (OSRCNode::SerialWrite { .. }, _) => PinType::SERIAL,
            (OSRCNode::SerialRead { itype, .. }, _) => *itype,
            (OSRCNode::GlobalVariableInput { name }, _) => PinType::UNDEFINED,
            (OSRCNode::GlobalVariableOutput { name }, _) => PinType::UNDEFINED,
            //TODO:
            //Query these from list!
            (OSRCNode::MathOperation { itype, .. }, _) => *itype,
            (OSRCNode::ApiOutput { node_name, itype }, _) => *itype,
            (OSRCNode::Invalid, _) => PinType::NONE, // _ => PinType::NONE,
            (OSRCNode::Constant { itype, .. }, _) => *itype,
            (OSRCNode::Comparator { .. }, _) => PinType::Bool,
            (OSRCNode::Multiplexer { itype, .. }, _) => *itype,
            (OSRCNode::PIController { .. }, _) => PinType::F32,
            (OSRCNode::VelEstimator { .. }, _) => PinType::F32,
        }
    }
    // pub fn output_value(&self, idx: usize) ->
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default, Debug)]
pub enum SerialDeviceReg {
    AsyncReg {
        update_cycles: u16,
    },
    CyclicReg {
        sync_node: bool,
        cyclic_index: u32,
    },
    // #[strum(disabled)]
    #[default]
    None,
}

impl PartialEq for SerialDeviceReg {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // (Self::AsyncReg { update_cycles: l_update_cycles }, Self::AsyncReg { update_cycles: r_update_cycles }) => l_update_cycles == r_update_cycles,
            // (Self::CyclicReg { sync_node: l_sync_node, cyclic_index: l_cyclic_index }, Self::CyclicReg { sync_node: r_sync_node, cyclic_index: r_cyclic_index }) => l_sync_node == r_sync_node && l_cyclic_index == r_cyclic_index,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    EnumIter,
    VariantArray,
    PartialEq,
    Display,
    Default,
)]
pub enum PinType {
    Bool,
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    F64,
    STRING,
    #[default]
    UNDEFINED,
    SERIAL,
    // #[strum(disabled)]
    NONE,
}

impl PinType {
    #[inline]
    pub fn can_connect(&self, other: &Self) -> bool {
        match (self, other) {
            (PinType::NONE, _) => false,
            (_, PinType::NONE) => false,
            (PinType::UNDEFINED, _) => true,
            (_, PinType::UNDEFINED) => true,
            (PinType::U8, PinType::I8)
            | (PinType::I8, PinType::U8)
            | (PinType::U16, PinType::I16)
            | (PinType::I16, PinType::U16)
            | (PinType::U32, PinType::I32)
            | (PinType::I32, PinType::U32) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }

    pub fn get_info(&self) -> PinInfo {
        match self {
            PinType::Bool => PinInfo::square().with_fill(Color32::RED),
            PinType::U8 => PinInfo::square().with_fill(Color32::ORANGE),
            PinType::U16 => PinInfo::square().with_fill(Color32::YELLOW),
            PinType::U32 => PinInfo::square().with_fill(Color32::GREEN),
            PinType::I8 => PinInfo::triangle().with_fill(Color32::ORANGE),
            PinType::I16 => PinInfo::triangle().with_fill(Color32::YELLOW),
            PinType::I32 => PinInfo::triangle().with_fill(Color32::GREEN),
            PinType::F32 => PinInfo::circle().with_fill(Color32::GREEN),
            PinType::F64 => PinInfo::circle().with_fill(Color32::BLUE),
            PinType::SERIAL => PinInfo::square().with_fill(Color32::LIGHT_BLUE),
            PinType::STRING => PinInfo::star().with_fill(Color32::PURPLE),
            PinType::UNDEFINED => PinInfo::star().with_fill(Color32::GREEN),
            PinType::NONE => PinInfo::star().with_fill(Color32::RED),
        }
    }

    pub fn from_size(size: usize) -> Self {
        match size {
            1 => PinType::Bool,
            8 => PinType::U8,
            16 => PinType::U16,
            32 => PinType::U32,
            _ => PinType::UNDEFINED,
        }
    }
    //
    // pub fn from_idx(i: &usize) -> Self {
    //     PinType::VARIANTS[*i]
    // }
    //
    // pub fn pretty_print(i: &PinType) -> String {
    //     format!("{:?}", i)
    // }

    // pub fn combo_select(mut ui: &mut Ui, mut current: &PinType) {
    //     ComboBox::from_label("Select Type:")
    //         .selected_text(PinType::pretty_print(current))
    //         .show_ui(ui, |ui| {
    //             for (i, t) in PinType::iter().enumerate() {
    //                 ui.selectable_value(&mut current, &t.clone(), PinType::pretty_print(&t));
    //             }
    //             //
    //         });
    // }
}

pub fn combo_select<T: Display + PartialEq + Clone>(
    mut ui: &mut Ui,
    label: String,
    mut current: &mut T,
    iter: impl Iterator<Item = T>,
) {
    ComboBox::from_label(label)
        .selected_text(format!("{}", current))
        .show_ui(ui, |ui| {
            for t in iter {
                // let mut c: &'a T = current;
                // let mut c: &T = &current;
                // ui.selectable_value(&mut current, &t, format!("{}", &t));
                if ui
                    .selectable_label(current == &t, format!("{}", &t))
                    .clicked()
                {
                    *current = t.clone();
                }
            }
        });
}
