#![allow(dead_code, unused, refining_impl_trait_internal)]
use std::{
    error::Error,
    fs::{self, File},
    io::{BufReader, BufWriter, Read},
    ops::RangeInclusive,
    path::PathBuf,
};

use chrono::NaiveDate;
use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{self, Assets},
};
use egui::{Color32, ComboBox, DragValue, Id, Stroke, Ui, Vec2, Vec2b, Widget, load::Bytes};
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{NodeLayout, PinInfo, SnarlStyle, SnarlViewer},
};
use file_structure::{FPGAProgram, FileInfo, GlobalVariable, Network, NetworkType};
use node::{OSRCNode, PinType};
use rfd::FileDialog;
use strum::{Display, IntoEnumIterator, IntoStaticStr};
use viewer::OSRCViewer;

pub mod file_structure;
pub mod node;
pub mod viewer;

fn main() {
    let d = discord_presense();

    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.inner_size = Some(Vec2::new(1920.0, 1080.0));
    native_options.persist_window = true;

    let _ = eframe::run_native(
        "OSRC Programmer",
        native_options,
        Box::new(|cc| Ok(Box::new(OsrcApp::new(cc)))),
    );
}

fn discord_presense() -> OSRCResult<()> {
    let mut cli = DiscordIpcClient::new("1378137813244969051")?;
    cli.connect()?;
    let assets = Assets::new().large_image("logo");
    let pl = activity::Activity::new().state("Programming a robot!");
    cli.set_activity(pl)?;
    cli.send_handshake()?;
    Ok(())
}

type OSRCResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Display, Clone)]
enum OSRCError {
    NoFileSelected,
    NoProgram,
}

impl Error for OSRCError {}

// #[derive(Default)]
struct OsrcApp {
    style: SnarlStyle,
    program: Option<FPGAProgram>,
    selected_net: usize,
    selected_var: usize,
    file: Option<PathBuf>,
}

impl OsrcApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = SnarlStyle::new();
        style.bg_frame = Some(egui::Frame {
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::same(2),
            corner_radius: egui::CornerRadius::ZERO,
            fill: egui::Color32::from_gray(95),
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        });
        // style.bg_pattern_stroke = Some(Stroke::new(100.0, Color32::PURPLE));

        // style.node_layout = NodeLayout::Basic;
        // Snarl has code for serdes on these
        OsrcApp {
            style,
            program: None,
            file: None,
            selected_net: 0,
            selected_var: 0,
        }
    }

    fn save_file(&mut self, location: Option<PathBuf>) -> OSRCResult<()> {
        let loc = location.or(self.file.clone());
        if loc.is_none() {
            return Err(Box::new(OSRCError::NoFileSelected));
        }
        if self.program.is_none() {
            return Err(Box::new(OSRCError::NoProgram));
        }

        let data = serde_yaml::to_string(&self.program.as_ref().unwrap())?;
        let file = File::create(loc.unwrap())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.program.as_ref().unwrap());
        Ok(())
    }

    fn load_file(&mut self) -> OSRCResult<()> {
        if self.file.is_none() {
            return Err(Box::new(OSRCError::NoFileSelected));
        }
        let p = self.file.clone().unwrap();
        if fs::exists(&p)? {
            let file = File::open(p)?;
            let rdr = BufReader::new(file);
            self.program = Some(serde_json::from_reader(rdr)?);
        } else {
            let info = FileInfo {
                date: chrono::Local::now().date_naive(),
                time: chrono::Local::now().time(),
                write_protected: false,
                fpga_config: String::new(),
            };
            let prg = FPGAProgram {
                info,
                networks: Vec::new(),
                set_node_vars: Vec::new(),
            };
            self.program = Some(prg);
        }
        Ok(())
    }
}

fn osrc_file_dialog() -> FileDialog {
    FileDialog::new().add_filter("OSRC File", &["osrc", "json"])
}

impl eframe::App for OsrcApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "File Options").show(
            ctx,
            |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        // New
                        // Open

                        // File Options
                        if self.file.is_some() {
                            if ui.button("Save").clicked() {
                                self.save_file(None);
                                ui.close_menu();
                            }

                            if ui.button("Save As").clicked() {
                                let loc = osrc_file_dialog().save_file();
                                if loc.is_some() {
                                    self.save_file(loc);
                                    ui.close_menu();
                                }
                            }

                            if ui.button("Save and close").clicked() {
                                self.save_file(None);
                                self.file = None;
                                self.program = None;

                                ui.close_menu();
                            }
                        } else {
                            if ui.button("New").clicked() {
                                let loc = osrc_file_dialog().save_file();
                                if loc.is_some() {
                                    let mut loc = loc.unwrap();
                                    loc.set_extension("json");  // use .json for now
                                    self.file = Some(loc);
                                    self.load_file();

                                    ui.close_menu();
                                }
                            }
                            if ui.button("Open").clicked() {
                                let loc = osrc_file_dialog().pick_file();
                                if loc.is_some() {
                                    self.file = loc;
                                    self.load_file();

                                    ui.close_menu();
                                }
                            }
                        }
                    });
                });
            },
        );

        egui::SidePanel::new(egui::panel::Side::Left, "Program Settings").show(ctx, |ui| {
            if self.program.is_none() {
                ui.label("No file loaded");
                return;
            }

            let mut prg = self.program.as_mut().unwrap();
            ui.vertical(|ui| {
                ui.label("File Info:");
                ui.end_row();
                ui.label(format!(
                    "Created on: {} at {}",
                    prg.info.date, prg.info.time
                ));
                ui.checkbox(&mut prg.info.write_protected, "Write Protect");
                ui.label("FPGA Config File:");
                ui.text_edit_singleline(&mut prg.info.fpga_config);
                ui.separator();
                ui.label("Global Variables");
                if prg.set_node_vars.len() > 0 {
                    if self.selected_var > prg.set_node_vars.len() {
                        self.selected_var = 0;
                    }
                    ComboBox::from_label("Select Variable")
                        .selected_text(format!(
                            "Variable: {}",
                            prg.set_node_vars
                                .get(self.selected_var)
                                .unwrap()
                                .name
                                .clone()
                        ))
                        .show_ui(ui, |ui| {
                            for (idx, net) in prg.set_node_vars.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_var, idx, net.name.clone());
                            }
                        });
                    let gvar = prg.set_node_vars.get_mut(self.selected_var).unwrap();
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut gvar.name);
                    ui.label("Type:");
                    node::combo_select(
                        ui,
                        "Select Type:".to_string(),
                        &mut gvar.pin_type,
                        PinType::iter(),
                    );
                    // ComboBox::from_label("Select Type")

                    //WARN: needs type add
                }
                ui.horizontal(|ui| {
                    if ui.button("Add Global Variable").clicked() {
                        prg.set_node_vars.push(GlobalVariable::new());
                    }
                    if ui.button("Remove Selected Variable").clicked() {
                        prg.set_node_vars.remove(self.selected_var);
                        self.selected_var = 0;
                    }
                });

                ui.separator();
                ui.label("Networks:");

                if prg.networks.len() > 0 {
                    if self.selected_net > prg.networks.len() {
                        self.selected_net = 0;
                    }

                    ComboBox::from_label("Select Network")
                        .selected_text(format!(
                            "Network: {}",
                            prg.networks.get(self.selected_net).unwrap().name.clone()
                        ))
                        .show_ui(ui, |ui| {
                            for (idx, net) in prg.networks.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_net, idx, net.name.clone());
                            }
                        });

                    let net = prg.networks.get_mut(self.selected_net).unwrap();
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut net.name);
                    ui.end_row();
                    ui.checkbox(&mut net.enabled, "Enabled");
                    ui.end_row();
                    ui.checkbox(
                        &mut net.dynamic_enable_starting,
                        "Dynamic Enable Starting",
                    );
                    ui.end_row();
                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        if ui
                            .selectable_label(net.net_type == NetworkType::Sync, "Sync")
                            .clicked()
                        {
                            net.net_type = NetworkType::Sync;
                        }
                        if ui
                            .selectable_label(net.net_type == NetworkType::Async, "Async")
                            .clicked()
                        {
                            net.net_type = NetworkType::Async;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Timeout: ");
                        DragValue::new(&mut net.timeout)
                            .range(RangeInclusive::new(0, 1 << 20))
                            .ui(ui);
                    });

                    ui.end_row();
                    ui.horizontal(|ui| {
                        ui.label("Update Cycle:");
                        DragValue::new(&mut net.update_cycle_trigger_count)
                            .range(RangeInclusive::new(0, 1 << 20))
                            .ui(ui);
                    });

                    ui.end_row();
                    ui.horizontal(|ui| {
                        ui.label("Execution Index:");
                        DragValue::new(&mut net.execution_index)
                            .range(RangeInclusive::new(0, 1 << 20))
                            .ui(ui);
                    });
                }

                ui.horizontal(|ui| {
                    if ui.button("Add Network").clicked() {
                        prg.networks.push(Network::new());
                    }
                    if ui.button("Remove Selected Network").clicked() {
                        prg.networks.remove(self.selected_net);
                        self.selected_net = 0;
                    }
                });
            });
            // egui::containers::Frame::canvas(ui.style()).show(ui, |ui| {});
            // ui.ctx().request_repaint();
            // ui.label("Hello!");
            // ui.image(frame);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if (self.program.is_some()) {
                let mut prog = self.program.as_mut().unwrap();
                let mut net = prog.networks.get_mut(self.selected_net);
                if net.is_some() && self.file.is_some() {
                    let id_name = format!(
                        "SnarlWindow {} {}",
                        self.selected_net,
                        self.file.clone().unwrap().to_str().unwrap()
                    );
                    net.unwrap()
                        .nodes
                        .show(&mut OSRCViewer, &self.style, Id::new(id_name), ui);
                } else {
                    ui.label("Network with given Id not found!");
                }
            } else {
                // No file loaded
            }
            // self.snarl
            //     .show(&mut OSRCViewer, &self.style, Id::new("snarl-window"), ui);        });

            // egui::Window::new("Window!")
            //     .resizable(Vec2b::new(false, false))
            //     .show(ctx, |ui| {
            //         ui.label("Hello!");
        });
    }
}
