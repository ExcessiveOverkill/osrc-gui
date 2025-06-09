use std::{collections::btree_map::Range, ops::RangeInclusive};

use convert_case::Casing;
use egui::{TextEdit, Ui, Widget, emath::Numeric};
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, SnarlViewer},
};
use strum::IntoEnumIterator;

use crate::node::{
    BinaryOperation, GateType, MathOperation, NaryOperation, OSRCNode, PinType, SerialDeviceReg,
    UnaryOperation, ValueCompare, combo_select,
};

fn drag_bar<Num: Numeric>(
    mut ui: &mut Ui,
    mut value: &mut Num,
    range: Option<RangeInclusive<Num>>,
) {
    let mut dg = egui::DragValue::new(value);
    if range.is_some() {
        dg = dg.range(range.unwrap());
    }
    dg.ui(ui);
}

fn opt_drag<Num: Numeric + Default>(
    mut ui: &mut Ui,
    mut value: &mut Option<Num>,
    range: Option<RangeInclusive<Num>>,
    name: String,
) {
    match value {
        Some(c) => {
            drag_bar(ui, c, range);
            if ui.button(format!("Remove {}:", name)).clicked() {
                *value = None;
            }
        }
        None => {
            if ui.button(format!("Add {}:", name)).clicked() {
                *value = Some(Num::default());
            }
        }
    }
}

fn pintype_sel(mut ui: &mut Ui, mut itype: &mut PinType, label: String) {
    combo_select(ui, label, itype, PinType::iter());
}

//
// fn textbox(mut ui: &mut Ui, &mut String) {
//     ui.text_edit_singleline(text)
// }

pub struct OSRCViewer;
impl SnarlViewer<OSRCNode> for OSRCViewer {
    fn title(&mut self, node: &OSRCNode) -> String {
        node.name()
    }

    fn inputs(&mut self, node: &OSRCNode) -> usize {
        match node {
            OSRCNode::ApiInput { .. } => 0,
            OSRCNode::ApiOutput { .. } => 1,
            OSRCNode::BitwiseSplit { .. } => 1,
            OSRCNode::BitwiseJoin { num_bits } => *num_bits,
            OSRCNode::EdgeDelay { .. } => 1,
            OSRCNode::CycleDelay { .. } => 1,
            OSRCNode::Converter { .. } => 1,
            OSRCNode::SerialDevice { num_write, .. } => 1 + num_write,
            OSRCNode::SerialRead { .. } => 1,
            OSRCNode::SerialWrite { .. } => 1,
            OSRCNode::GlobalVariableInput { .. } => 1,
            OSRCNode::GlobalVariableOutput { .. } => 0,
            OSRCNode::LogicGate { gtype } => match gtype {
                GateType::AND(i) => *i,
                GateType::OR(i) => *i,
                GateType::NAND(i) => *i,
                GateType::NOR(i) => *i,
                GateType::XOR(i) => *i,
                GateType::NOT => 1,
            },
            OSRCNode::MathOperation { operator, .. } => match operator {
                crate::node::MathOperation::Nary(nary_operation, n) => *n,
                crate::node::MathOperation::BinaryOperation(binary_operation) => 2,
                crate::node::MathOperation::UnaryOperation(unary_operation) => 1,
                crate::node::MathOperation::Invalid => 0,
            },
            OSRCNode::Invalid => 0,
            OSRCNode::Comparator { .. } => 2,
            OSRCNode::Constant { .. } => 0,
            OSRCNode::Multiplexer { input_bits, .. } => input_bits + (1 << input_bits),
            OSRCNode::PIController { .. } => 1,
            OSRCNode::VelEstimator { .. } => 1,
        }
    }

    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut Snarl<OSRCNode>,
    ) -> PinInfo {
        snarl[pin.id.node].pin_type_input(pin.id.input).get_info()
        // PinInfo::circle()
        // todo!()
    }

    fn outputs(&mut self, node: &OSRCNode) -> usize {
        match node {
            OSRCNode::ApiInput { .. } => 1,
            OSRCNode::ApiOutput { .. } => 0,
            OSRCNode::BitwiseSplit { num_bits } => *num_bits,
            OSRCNode::BitwiseJoin { num_bits } => 1,
            OSRCNode::EdgeDelay { .. } => 1,
            OSRCNode::CycleDelay { .. } => 1,
            OSRCNode::Converter { .. } => 1,
            OSRCNode::SerialDevice { num_read, .. } => *num_read,
            OSRCNode::SerialRead { .. } => 1,
            OSRCNode::SerialWrite { .. } => 1,
            OSRCNode::GlobalVariableInput { name } => 0,
            OSRCNode::GlobalVariableOutput { name } => 1,
            OSRCNode::LogicGate { .. } => 1,
            OSRCNode::MathOperation { .. } => 1,
            OSRCNode::Invalid => 0,
            OSRCNode::Constant { .. } => 1,
            OSRCNode::Comparator { .. } => 1,
            OSRCNode::Multiplexer { .. } => 1,
            OSRCNode::PIController { .. } => 1,
            OSRCNode::VelEstimator { .. } => 1,
        }
    }

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut Snarl<OSRCNode>,
    ) -> PinInfo {
        snarl[pin.id.node].pin_type_output(pin.id.output).get_info()
        // PinInfo::circle()
    }

    fn has_node_menu(&mut self, _node: &OSRCNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        _u: f32,
        snarl: &mut Snarl<OSRCNode>,
    ) {
        ui.label("Node menu");
        ui.separator();

        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
    }

    fn has_graph_menu(&mut self, pos: egui::Pos2, snarl: &mut Snarl<OSRCNode>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        scale: f32,
        snarl: &mut Snarl<OSRCNode>,
    ) {
        ui.label("Add Node");

        for node in OSRCNode::iter() {
            if ui.button(node.name()).clicked() {
                snarl.insert_node(pos, node);
                ui.close_menu();
            }
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<OSRCNode>) {
        if snarl[from.id.node]
            .pin_type_output(from.id.output)
            .can_connect(&snarl[to.id.node].pin_type_input(to.id.input))
        {
            for &remote in &to.remotes {
                snarl.disconnect(remote, to.id);
            }
            snarl.connect(from.id, to.id);
        }
    }

    fn has_body(&mut self, node: &OSRCNode) -> bool {
        true
    }

    fn show_body(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        scale: f32,
        snarl: &mut Snarl<OSRCNode>,
    ) {
        ui.vertical(|ui| {
            match snarl.get_node_mut(node).unwrap() {
                OSRCNode::ApiInput {
                    min,
                    max,
                    default,
                    timeout,
                    itype,
                    node_name,
                } => {
                    egui::TextEdit::singleline(node_name).ui(ui);
                    //let range = Some(RangeInclusive::new(0.0, f32::MAX));
                    let range = None;
                    opt_drag(ui, min, range.clone(), "Min".to_string());
                    opt_drag(ui, max, range.clone(), "Max".to_string());
                    opt_drag(ui, default, range.clone(), "Default".to_string());
                    opt_drag(ui, timeout, range.clone(), "Timeout".to_string());
                    pintype_sel(ui, itype, "Type".to_string());
                }
                OSRCNode::BitwiseSplit { num_bits } => {
                    ui.label("Number of bits: ");
                    egui::DragValue::new(num_bits)
                        .range(RangeInclusive::new(2, 32))
                        .ui(ui);
                }
                OSRCNode::BitwiseJoin { num_bits } => {
                    ui.label("Number of bits: ");
                    egui::DragValue::new(num_bits)
                        .range(RangeInclusive::new(2, 32))
                        .ui(ui);
                }
                OSRCNode::EdgeDelay { cycles, rising_edge, falling_edge} => {
                    ui.label("Delay Cycles: ");
                    egui::DragValue::new(cycles)
                        .range(RangeInclusive::new(1, 0xFFFF))
                        .ui(ui);

                    ui.horizontal(|ui| {
                        ui.radio_value(rising_edge, true, "Rising");
                        ui.radio_value(rising_edge, false, "Falling");
                    });
                }
                OSRCNode::CycleDelay { cycles, itype} => {
                    pintype_sel(ui, itype, "Type".to_string());
                    ui.label("Delay Cycles: ");
                    egui::DragValue::new(cycles)
                        .range(RangeInclusive::new(1, 16))
                        .ui(ui);
                }
                OSRCNode::Converter {
                    input_type,
                    output_type,
                    direct_mode,
                    input_max,
                    input_min,
                    output_max,
                    output_min,
                    invert,
                } => {
                    ui.label("Input type:");
                    pintype_sel(ui, input_type, "Input Type".to_string());
                    ui.label("Output type:");
                    pintype_sel(ui, output_type, "Output Type".to_string());

                    ui.checkbox(direct_mode, "Direct Mode");
                    
                    if *direct_mode == false {
                        ui.end_row();
                        ui.label("Input Min:");
                        egui::TextEdit::singleline(input_min).ui(ui);
                        ui.end_row();
                        ui.label("Input Max:");
                        egui::TextEdit::singleline(input_max).ui(ui);
                        ui.end_row();
                        ui.label("Output Min:");
                        egui::TextEdit::singleline(output_min).ui(ui);
                        ui.end_row();
                        ui.label("Output Max:");
                        egui::TextEdit::singleline(output_max).ui(ui);
                        ui.end_row();
                        ui.checkbox(invert, "Invert");
                    }
                }
                OSRCNode::SerialDevice {
                    enabled,
                    addr,
                    timeout,
                    descriptor,
                    num_read,
                    num_write,
                    node_name
                } => {
                    ui.label("Node Name: ");
                    egui::TextEdit::singleline(node_name).ui(ui);


                    ui.label("File Descriptor:");
                    egui::TextEdit::singleline(descriptor).ui(ui);

                    ui.end_row();
                    ui.label("Num Reading: ");
                    egui::DragValue::new(num_read)
                        .range(RangeInclusive::new(0, 32))
                        .ui(ui);

                    ui.end_row();
                    ui.label("Num Writing: ");
                    egui::DragValue::new(num_write)
                        .range(RangeInclusive::new(0, 32))
                        .ui(ui);
                }
                OSRCNode::SerialRead { name, dev, itype }
                | OSRCNode::SerialWrite { name, dev, itype } => {
                    ui.label("Register Name: ");
                    egui::TextEdit::singleline(name).ui(ui);
                    ui.end_row();
                    pintype_sel(ui, itype, "Type".to_string());
                    ui.radio_value(dev, SerialDeviceReg::AsyncReg { update_cycles: 1 }, "Async");
                    ui.radio_value(
                        dev,
                        SerialDeviceReg::CyclicReg {
                            sync_node: false,
                            cyclic_index: 1,
                        },
                        "Cyclic",
                    );

                    ui.end_row();
                    match dev {
                        SerialDeviceReg::AsyncReg { update_cycles } => {
                            ui.label("Update Rate: ");
                            egui::DragValue::new(update_cycles)
                                .range(RangeInclusive::new(0, 1 << 16))
                                .ui(ui);
                        }
                        SerialDeviceReg::CyclicReg {
                            sync_node,
                            cyclic_index,
                        } => {
                            ui.checkbox(sync_node, "Sync Node");

                            ui.end_row();
                            ui.label("Update Rate: ");
                            egui::DragValue::new(cyclic_index)
                                .range(RangeInclusive::new(0, 1 << 16))
                                .ui(ui);
                            //
                        }
                        SerialDeviceReg::None => {}
                    }
                }
                OSRCNode::GlobalVariableInput { name } => {
                    egui::TextEdit::singleline(name).show(ui);
                }
                OSRCNode::GlobalVariableOutput { name } => {
                    egui::TextEdit::singleline(name).show(ui);
                }
                OSRCNode::ApiOutput { node_name, itype } => {
                    TextEdit::singleline(node_name).show(ui);
                    pintype_sel(ui, itype, "Type".to_string());
                }
                OSRCNode::LogicGate { gtype } => {
                    combo_select(ui, "Select Gate".to_string(), gtype, GateType::iter());
                    match gtype {
                        GateType::AND(i)
                        | GateType::OR(i)
                        | GateType::NAND(i)
                        | GateType::NOR(i)
                        | GateType::XOR(i) => {
                            drag_bar(ui, i, Some(RangeInclusive::new(2, 16)));
                        }
                        GateType::NOT => {}
                    }
                }
                OSRCNode::MathOperation { itype, operator } => {
                    pintype_sel(ui, itype, "Type".to_string());
                    combo_select(
                        ui,
                        "Select:".to_string(),
                        operator,
                        MathOperation::iter().filter(|o| *o != MathOperation::Invalid),
                    );
                    match operator {
                        MathOperation::Nary(nary_operation, i) => {
                            combo_select(
                                ui,
                                "N-Ary Operator".to_string(),
                                nary_operation,
                                NaryOperation::iter(),
                            );
                            drag_bar(ui, i, Some(RangeInclusive::new(2, 16)));
                        }
                        MathOperation::BinaryOperation(binary_operation) => combo_select(
                            ui,
                            "Binary Operator".to_string(),
                            binary_operation,
                            BinaryOperation::iter(),
                        ),
                        MathOperation::UnaryOperation(unary_operation) => combo_select(
                            ui,
                            "Unary Operator".to_string(),
                            unary_operation,
                            UnaryOperation::iter(),
                        ),
                        MathOperation::Invalid => {}
                    }
                }
                OSRCNode::Invalid => unimplemented!(),
                OSRCNode::Constant { itype, value, node_name} => {
                    ui.label("Node name: ");
                    egui::TextEdit::singleline(node_name).ui(ui);
                    pintype_sel(ui, itype, "Type".to_string());
                    TextEdit::singleline(value).show(ui);
                }
                OSRCNode::Comparator { itype, comparison } => {
                    pintype_sel(ui, itype, "Type".to_string());
                    combo_select(ui, "Cmp".to_string(), comparison, ValueCompare::iter());
                }
                OSRCNode::Multiplexer { itype, input_bits } => {
                    pintype_sel(ui, itype, "Type".to_string());
                    drag_bar(ui, input_bits, Some(RangeInclusive::new(1, 8)));
                }
                OSRCNode::PIController { p, i, i_limit, output_min, output_max, node_name} => {
                    ui.label("Node name: ");
                    egui::TextEdit::singleline(node_name).ui(ui);
                    ui.label("P gain");
                    egui::DragValue::new(p)
                        .range(RangeInclusive::new(0.0, f32::MAX))
                        .ui(ui);
                    ui.label("I gain ");
                    egui::DragValue::new(i)
                        .range(RangeInclusive::new(0.0, f32::MAX))
                        .ui(ui);
                    ui.label("I limit ");
                    egui::DragValue::new(i_limit)
                        .range(RangeInclusive::new(0.0, f32::MAX))
                        .ui(ui);
                    ui.label("Min Output: ");
                    egui::DragValue::new(output_min)
                        .range(RangeInclusive::new(f32::MIN, f32::MAX))
                        .ui(ui);
                    ui.label("Max Output: ");
                    egui::DragValue::new(output_max)
                        .range(RangeInclusive::new(f32::MIN, f32::MAX))
                        .ui(ui);
                }
                OSRCNode::VelEstimator { itype, alpha} => {
                    ui.label("Alpha: ");
                    egui::DragValue::new(alpha)
                        .range(RangeInclusive::new(0, 1))
                        .ui(ui);
                }
            }
        });
        // ui.label("Body!");
    }
}
