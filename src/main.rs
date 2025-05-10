// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

use common::GuiAnalysis;
use eframe::egui::{self, RichText, Ui};
use leaflets::{LeafletClassification, LeafletClassificationParams};

mod analysis_types;
mod common;
mod estimate_error;
mod frame_selection;
mod leaflets;
mod membrane_normal;
mod ordermaps;
mod other_options;

pub const GUIORDER_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([465.0, 640.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        &format!("guiorder v{}", GUIORDER_VERSION),
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::<GuiAnalysis>::default())
        }),
    )
}

impl eframe::App for GuiAnalysis {
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

                    ui.horizontal(|ui| {
                        ui.vertical_centered(|ui| {
                            let mut input_yaml = if ui.button("📁 Import from YAML").on_hover_ui(|ui| {
                                ui.label("Load a YAML configuration file.");
                            }).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    Some(path.display().to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                        });
                    });

                    ui.separator();

                    GuiAnalysis::specify_input_file(
                        &mut self.structure,
                        ui,
                        "Structure:   ",
                        "Path to a file containing the structure of the system.",
                        true,
                    );
                    GuiAnalysis::specify_multiple_input_files(
                        &mut self.trajectory,
                        ui,
                        "Trajectory:  ",
                        "Path to a file containing the trajectory to analyze. Provide multiple files by clicking the '+' button or by selecting them interactively.",
                        true,
                    );
                    GuiAnalysis::specify_output_file(
                        &mut self.output.output_yaml,
                        ui,
                        "Output YAML: ",
                        "Path to an output YAML file where the full results of the analysis will be saved.",
                        true,
                    );

                    ui.separator();
                    self.specify_analysis_type(ui);
                    ui.separator();

                    self.specify_advanced_input(ui);
                    self.specify_advanced_output(ui);
                    self.specify_frame_selection(ui);
                    self.specify_membrane_normal(ui);
                    self.specify_leaflet_classification(ui);
                    self.specify_ordermaps(ui);
                    self.specify_estimate_error(ui);
                    self.specify_other_options(ui);

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add_space(54.0);
                        if ui.button("📁 Export to YAML").on_hover_ui(|ui| {
                            ui.label("Export the analysis options into a YAML configuration file.");
                        }).clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                // todo; convert and export parameters
                            }
                        }

                        ui.add_space(54.0);
                        ui.separator();
                        ui.add_space(46.0);

                        let hint = if self.other_params.n_threads >= 2 {
                            format!("Perform the analysis using {} threads.", self.other_params.n_threads)
                        } else {
                            format!("Perform the analysis using 1 thread.")
                        };

                        if Self::smart_button(
                            ui,
                            self.check_sanity(),
                            "🔥 Run the analysis",
                            &hint,
                            "Cannot run the analysis because some options are missing."
                        ).clicked() {
                            // todo; convert and run
                        };
                    });

                    ui.separator();
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
    }
}
