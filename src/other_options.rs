// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Specification of parameters that do not fit elsewhere.

use eframe::egui::{DragValue, Ui};

use crate::GuiAnalysis;

impl GuiAnalysis {
    pub(super) fn specify_other_options(&mut self, ui: &mut Ui) {
        if self.n_threads == 0 {
            self.n_threads = 1;
        }

        if self.min_samples == 0 {
            self.min_samples = 1;
        }

        Self::collapsing_with_warning(ui, "Other options", false, true, |ui| {
            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Minimum samples:   ",
                    "Minimum number of samples collected for each bond required to calculate order parameter for it.",
                );
    
                ui.add(
                    DragValue::new(&mut self.min_samples)
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
                    DragValue::new(&mut self.n_threads)
                        .speed(0.05)
                        .range(1..=usize::MAX),
                );
            });

            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Overwrite: ",
                    "Check the box if you want the output files to overwrite existing files with the same names instead of backing up the old files.",
                );
    
                ui.checkbox(&mut self.overwrite, "");
            });

            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Silent:    ",
                    "Check the box if you want no information about the progress of the analysis to be reported.",
                );
    
                ui.checkbox(&mut self.silent, "");
            });

            
        });
    }
}
