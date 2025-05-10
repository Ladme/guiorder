// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Specification of parameters that do not fit elsewhere.

use eframe::egui::{self, Color32, DragValue, RichText, Ui};

use crate::GuiAnalysis;

/// Parameters that do not fit elsewhere.
#[derive(Debug, Clone)]
pub(crate) struct OtherParams {
    min_samples: usize,
    pub n_threads: usize,
    handle_pbc: bool,
    overwrite: bool,
    silent: bool,
}

impl Default for OtherParams {
    fn default() -> Self {
        Self {
            min_samples: 1,
            n_threads: 1,
            handle_pbc: true,
            overwrite: false,
            silent: false,
        }
    }
}

impl GuiAnalysis {
    /// Specify parameters that do not fit elsewhere.
    pub(super) fn specify_other_options(&mut self, ui: &mut Ui) {
        Self::collapsing_with_warning(ui, "Other options", false, true, |ui| {
            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Minimum samples:   ",
                    "Minimum number of samples collected for each bond required to calculate order parameter for it.",
                );
    
                ui.add(
                    DragValue::new(&mut self.other_params.min_samples)
                        .speed(5)
                        .range(1..=usize::MAX),
                );
            });

            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Number of threads: ",
                    "Number of threads used to perform the analysis.",
                );
    
                ui.add(
                    DragValue::new(&mut self.other_params.n_threads)
                        .speed(0.05)
                        .range(1..=usize::MAX),
                );
            });

            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Handle PBC: ",
                    "Check this box if you want the program to automatically handle periodic boundary conditions.",
                );
                
                ui.checkbox(&mut self.other_params.handle_pbc, "");

                if self.other_params.handle_pbc {
                    ui.label(
                        RichText::new("simulation box must be orthogonal")
                            .font(egui::FontId::proportional(10.0)),
                    );
                } else {
                    ui.label(
                        RichText::new("lipid molecules must be whole!")
                            .font(egui::FontId::proportional(10.0))
                            .color(Color32::from_rgb(200, 150, 0)),
                    );
                }
                
            });

            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Overwrite:  ",
                    "Check the box if you want the output files to overwrite existing files with the same names instead of backing up the old files.",
                );
    
                ui.checkbox(&mut self.other_params.overwrite, "");
            });

            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Silent:     ",
                    "Check the box if you want no information about the progress of the analysis to be reported.",
                );
    
                ui.checkbox(&mut self.other_params.silent, "");
            });

            
        });
    }
}
