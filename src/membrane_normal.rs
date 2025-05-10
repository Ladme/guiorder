// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Parameters related to membrane normal.

use eframe::egui::{self, RichText, Ui};

use crate::{common::MembraneNormal, GuiAnalysis};

/// Parameters for dynamic membrane normal calculations.
#[derive(Debug, Clone)]
pub(crate) struct DynamicNormalParams {
    heads: String,
    radius: f32,
}

impl Default for DynamicNormalParams {
    fn default() -> Self {
        DynamicNormalParams {
            heads: String::new(),
            radius: 2.0,
        }
    }
}

impl GuiAnalysis {
    /// Specify the global membrane normal or parameters for its calculation.
    pub(super) fn specify_membrane_normal(&mut self, ui: &mut Ui) {
        Self::collapsing_with_warning(
            ui,
            "Membrane normal",
            false,
            self.check_membrane_normal_sanity(),
            |ui| {
                ui.horizontal(|ui| {
                    Self::label_with_hint(
                        ui,
                        "Membrane normal: ",
                        "Direction of the membrane normal.",
                    );

                    ui.radio_value(&mut self.membrane_normal, MembraneNormal::X, "x");
                    ui.radio_value(&mut self.membrane_normal, MembraneNormal::Y, "y");
                    ui.radio_value(&mut self.membrane_normal, MembraneNormal::Z, "z");
                    ui.radio_value(
                        &mut self.membrane_normal,
                        MembraneNormal::Dynamic,
                        "dynamic",
                    );
                });

                if self.membrane_normal == MembraneNormal::Dynamic {
                    ui.vertical(|ui| {
                    Self::specify_string(
                        &mut self.dynamic_normal_params.heads,
                        ui,
                        "Lipid heads: ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );

                    ui.horizontal(|ui| {
                        let label = Self::label_with_hint(
                            ui,
                            "Radius:      ",
                            "Radius of the scanning sphere for identification of nearby lipid heads."
                        );

                        ui.add(
                            egui::DragValue::new(
                                &mut self.dynamic_normal_params.radius,
                            )
                            .speed(0.1)
                            .range(0.0..=f32::MAX)
                            .suffix(" nm"),
                        )
                        .labelled_by(label.id);
                    })
                });
                }
            },
        );
    }

    /// Check that all required options for membrane normal specification have been provided.
    pub(super) fn check_membrane_normal_sanity(&self) -> bool {
        match self.membrane_normal {
            MembraneNormal::Dynamic => !self.dynamic_normal_params.heads.is_empty(),
            _ => true,
        }
    }
}
