// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use colored::Colorize;
use common::GuiAnalysis;
use eframe::egui::{self, RichText, Ui};
use gorder::colog_info;
use leaflets::{LeafletClassification, LeafletClassificationParams};
use window::Windows;

mod analysis_types;
mod common;
mod convert;
mod error;
mod estimate_error;
mod frame_selection;
mod geometry;
mod leaflets;
mod membrane_normal;
mod ordermaps;
mod other_options;
mod window;

pub const GUIORDER_VERSION: &str = env!("CARGO_PKG_VERSION");
const LINE_SPACING: f32 = 10.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([465.0, 640.0])
            .with_resizable(false),
        ..Default::default()
    };

    colog::init();

    eframe::run_native(
        &format!("guiorder v{}", GUIORDER_VERSION),
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::from(GuiOrderApp::default()))
        }),
    )
}

/// Structure handling the entire application.
#[derive(Debug, Default)]
pub(crate) struct GuiOrderApp {
    analysis: GuiAnalysis,
    windows: Windows,
    /// Analysis running?
    running: Arc<Mutex<bool>>,
    thread_handle: Mutex<Option<JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>>,
}

impl eframe::App for GuiOrderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            RichText::new(format!("guiorder v{}", GUIORDER_VERSION))
                                .font(egui::FontId::monospace(20.0)),
                        );
                        ui.heading(
                            RichText::new(format!(
                                "Graphical User Interface for gorder (v{})",
                                gorder::GORDER_VERSION
                            ))
                            .font(egui::FontId::monospace(15.0)),
                        );
                    });

                    ui.add_space(20.0);
                    self.import_yaml(ui);
                    ui.separator();

                    GuiAnalysis::specify_input_file(
                        &mut self.analysis.structure,
                        ui,
                        "Structure:   ",
                        "Path to a file containing the structure of the system.",
                        true,
                    );
                    GuiAnalysis::specify_multiple_input_files(
                        &mut self.analysis.trajectory,
                        ui,
                        "Trajectory:  ",
                        "Path to a file containing the trajectory to analyze. Provide multiple files by clicking the '+' button or by selecting them interactively.",
                        true,
                    );
                    GuiAnalysis::specify_output_file(
                        &mut self.analysis.output.output_yaml,
                        ui,
                        "Output YAML: ",
                        "Path to an output YAML file where the full results of the analysis will be saved.",
                        true,
                    );

                    ui.separator();
                    self.analysis.specify_analysis_type(ui);
                    ui.separator();

                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_advanced_input(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_advanced_output(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_frame_selection(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_membrane_normal(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_leaflet_classification(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_ordermaps(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_geometry(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_estimate_error(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_other_options(ui);
                    ui.add_space(LINE_SPACING);

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add_space(54.0);
                        if GuiAnalysis::smart_button(
                            ui,
                            self.analysis.check_sanity(),
                            false,
                            "📁 Export to YAML",
                            "Export analysis options into a YAML configuration file.",
                            "Cannot export analysis options because some are missing.",
                            "This should never appear.",
                        ).clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                self.export_to_yaml(path);
                            }
                        }

                        ui.add_space(54.0);
                        ui.separator();
                        ui.add_space(46.0);

                        let hint = if self.analysis.other_params.n_threads >= 2 {
                            format!("Perform the analysis using {} threads.", self.analysis.other_params.n_threads)
                        } else {
                            format!("Perform the analysis using 1 thread.")
                        };

                        if GuiAnalysis::smart_button(
                            ui,
                            self.analysis.check_sanity(),
                            *self.running.lock().unwrap(),
                            "🔥 Run the analysis",
                            &hint,
                            "Cannot run the analysis because some options are missing.", 
                            "Analysis is already running."
                        ).clicked() {
                            self.run_analysis();
                        };
                    });

                    ui.separator();

                    // display that the analysis is running
                    if *self.running.lock().unwrap() {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.spinner();
                            ui.label(
                                RichText::new("Analysis is running. See the terminal for more details.")
                                    .font(egui::FontId::monospace(12.0))
                            );
                        });
                    // check for errors during the analysis
                    } else {
                        let handle = self.thread_handle.lock().unwrap().take();
                        match handle {
                            None => (), // no result, do nothing
                            Some(handle) => {
                                match handle.join().unwrap() {
                                    Ok(_) => {
                                        Self::display_result(true, self.analysis.other_params.silent);
                                        self.open_success_window("Analysis finished successfully.");
                                    }
                                    Err(e) => {
                                        log::error!("{}", e);
                                        Self::display_result(false, self.analysis.other_params.silent);
                                        self.open_error_window(e);
                                    }
                                }
                            }
                        }
                    }

                    // render windows
                    self.windows.render(ctx);
                });
        });
    }
}

/// Paths to all specified output files.
#[derive(Debug, Clone, Default)]
struct OutputFiles {
    output_yaml: String,
    output_csv: String,
    output_tab: String,
    output_xvg: String,
}

impl From<&gorder::input::Analysis> for OutputFiles {
    fn from(value: &gorder::input::Analysis) -> Self {
        Self {
            output_yaml: value.output_yaml().clone().unwrap_or(String::new()),
            output_csv: value.output_csv().clone().unwrap_or(String::new()),
            output_tab: value.output_tab().clone().unwrap_or(String::new()),
            output_xvg: value.output_xvg().clone().unwrap_or(String::new()),
        }
    }
}

impl GuiOrderApp {
    /// Import parameters from a YAML file.
    fn import_yaml(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical_centered(|ui| {
                let input_yaml = if ui
                    .button("📁 Import from YAML")
                    .on_hover_ui(|ui| {
                        ui.label("Load a YAML configuration file.");
                    })
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().set_directory(".").pick_file() {
                        Some(path.display().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(input) = input_yaml {
                    match gorder::input::Analysis::from_file(&input) {
                        Err(e) => self.open_error_window(Box::from(e)),
                        Ok(analysis) => match analysis.try_into() {
                            Err(e) => self.open_error_window(Box::from(e)),
                            Ok(converted) => {
                                self.analysis = converted;
                            }
                        },
                    }
                }
            });
        });
    }

    /// Convert the GuiAnalysis to gorder analysis structure and run the analysis.
    fn run_analysis(&mut self) {
        let converted = match gorder::input::Analysis::try_from(&self.analysis) {
            Err(e) => {
                self.open_error_window(Box::from(e));
                return;
            }
            Ok(x) => x,
        };

        if !self.analysis.other_params.silent {
            log::set_max_level(log::LevelFilter::Info);
            let header = format!(">>> GORDER v{} <<<", gorder::GORDER_VERSION).bold();
            println!("\n{}\n", header);
        } else {
            log::set_max_level(log::LevelFilter::Error);
        }

        colog_info!(
            "Analysis parameters supplied by {}.",
            format!("guiorder v{}", GUIORDER_VERSION)
        );

        let is_running = Arc::clone(&self.running);
        *self.running.lock().unwrap() = true;

        let handle = std::thread::spawn(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                match converted.run() {
                    Err(e) => {
                        *is_running.lock().unwrap() = false;
                        Err(e)
                    }
                    Ok(results) => match results.write() {
                        Err(e) => {
                            *is_running.lock().unwrap() = false;
                            Err(Box::from(e))
                        }
                        Ok(_) => {
                            *is_running.lock().unwrap() = false;
                            Ok(())
                        }
                    },
                }
            },
        );

        *self.thread_handle.lock().unwrap() = Some(handle);
    }

    /// Convert the GuiAnalysis to gorder analysis structure and export it to an output yaml file.
    fn export_to_yaml(&mut self, output: PathBuf) {
        let converted = match gorder::input::Analysis::try_from(&self.analysis) {
            Err(e) => {
                self.open_error_window(Box::from(e));
                return;
            }
            Ok(x) => x,
        };

        let file = match File::create(&output) {
            Err(e) => {
                self.open_error_window(Box::from(e));
                return;
            }
            Ok(x) => x,
        };
        let mut writer = BufWriter::new(file);

        writeln!(
            writer,
            "# Analysis options generated by 'guiorder v{}'.",
            GUIORDER_VERSION
        )
        .unwrap();

        match serde_yaml::to_writer(&mut writer, &converted) {
            Err(e) => {
                self.open_error_window(Box::from(e));
                return;
            }
            Ok(_) => self.open_success_window(&format!(
                "Successfully exported analysis options into a configuration YAML file '{}'.",
                output.to_str().unwrap()
            )),
        };
    }

    /// Display the result of the analysis.
    fn display_result(result: bool, silent: bool) {
        if silent {
            return;
        }

        match result {
            true => {
                let prefix = format!(
                    "{}{}{}",
                    "[".to_string().blue().bold(),
                    "✔".to_string().bright_green().bold(),
                    "]".to_string().blue().bold()
                );
                let message = "ANALYSIS COMPLETED".to_string().bright_green().bold();
                println!("{} {}\n", prefix, message);
            }
            false => {
                let prefix = format!(
                    "{}{}{}",
                    "[".to_string().blue().bold(),
                    "✖".to_string().red().bold(),
                    "]".to_string().blue().bold()
                );
                let message = "ANALYSIS FAILED".to_string().red().bold();
                println!("{} {}\n", prefix, message);
            }
        }
    }
}

impl GuiAnalysis {
    /// Specify optional paths to a bonds file and an NDX file.
    fn specify_advanced_input(&mut self, ui: &mut Ui) {
        ui.collapsing(
            RichText::new("Advanced input").font(egui::FontId::monospace(12.0)),
            |ui| {
                Self::specify_input_file(
                &mut self.bonds,
                ui,
                "Bonds file:   ",
                "Path to a file containing information about the bonds of the system. (Optional)",
                false,
            );
                Self::specify_input_file(
                &mut self.ndx,
                ui,
                "NDX file:     ",
                "Path to an NDX file containing the groups associated with the system. (Optional)",
                false,
            );
            },
        );
    }

    /// Specify paths to CSV, Table, and XVG output.
    fn specify_advanced_output(&mut self, ui: &mut Ui) {
        ui.collapsing(
            RichText::new("Advanced output").font(egui::FontId::monospace(12.0)),
            |ui| {
                Self::specify_output_file(
                    &mut self.output.output_csv,
                    ui,
                    "Output CSV:   ",
                    "Path to an output CSV file where the results will be saved. (Optional)",
                    false,
                );
                Self::specify_output_file(
                    &mut self.output.output_tab,
                    ui,
                    "Output Table: ",
                    "Path to an output \"table\" file where the results will be saved. (Optional)",
                    false,
                );
                Self::specify_output_file(
                &mut self.output.output_xvg,
                ui,
                "Output XVG:   ",
                "Filename pattern for output XVG files where the results will be saved. (Optional)",
                false,
            );
            },
        );
    }

    /// Check that all options required for the analysis have been provided.
    fn check_sanity(&self) -> bool {
        self.check_leaflets_sanity()
            && self.check_analysis_params_sanity()
            && !self.structure.is_empty()
            && !self.trajectory.iter().any(|file| file.is_empty())
            && !self.output.output_yaml.is_empty()
            && self.check_membrane_normal_sanity()
            && self.check_ordermaps_sanity()
            && self.check_geometry_sanity()
    }
}
