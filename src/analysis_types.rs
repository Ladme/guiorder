// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Analysis types and their parameters.

use eframe::egui::Ui;

use crate::GuiAnalysis;

/// Type of analysis to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum AnalysisType {
    #[default]
    AAOrder,
    UAOrder,
    CGOrder,
}

/// Parameters for calculating AA order.
#[derive(Debug, Clone, Default)]
struct AAParams {
    heavy_atoms: String,
    hydrogens: String,
}

/// Parameters for calculating CG order.
#[derive(Debug, Clone, Default)]
struct CGParams {
    beads: String,
}

/// Parameters for calculating UA order.
#[derive(Debug, Clone, Default)]
struct UAParams {
    saturated: String,
    unsaturated: String,
    ignore: String,
}

/// Parameters for all analysis types.
#[derive(Debug, Clone, Default)]
pub(crate) struct AnalysisTypeParams {
    aa_params: AAParams,
    ua_params: UAParams,
    cg_params: CGParams,
}

impl GuiAnalysis {
    /// Specify the type of analysis to perform and parameters for it.
    pub(super) fn specify_analysis_type(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            Self::label_with_hint(
                ui,
                "Analysis type: ",
                "Type of order parameters to calculate.",
            );

            ui.radio_value(&mut self.analysis_type, AnalysisType::AAOrder, "atomistic");
            ui.radio_value(
                &mut self.analysis_type,
                AnalysisType::CGOrder,
                "coarse-grained",
            );
            ui.radio_value(
                &mut self.analysis_type,
                AnalysisType::UAOrder,
                "united-atom",
            );
        });

        ui.vertical(|ui| match self.analysis_type {
            AnalysisType::AAOrder => {
                Self::specify_string(
                    &mut self.analysis_type_params.aa_params.heavy_atoms,
                    ui,
                    "Heavy atoms: ",
                    "Selection of heavy atoms to be used in the analysis.",
                    true,
                );
                Self::specify_string(
                    &mut self.analysis_type_params.aa_params.hydrogens,
                    ui,
                    "Hydrogens:   ",
                    "Selection of hydrogens to be used in the analysis.",
                    true,
                );
            }

            AnalysisType::UAOrder => {
                Self::specify_string(
                    &mut self.analysis_type_params.ua_params.saturated,
                    ui,
                    "Saturated carbons:   ",
                    "Selection of saturated carbons to be used in the analysis.",
                    true,
                );
                Self::specify_string(
                    &mut self.analysis_type_params.ua_params.unsaturated,
                    ui,
                    "Unsaturated carbons: ",
                    "Selection of unsaturated carbons to be used in the analysis.",
                    true,
                );
                Self::specify_string(
                    &mut self.analysis_type_params.ua_params.ignore,
                    ui,
                    "Ignore:              ",
                    "Selection of atoms to be ignored. (Optional)",
                    false,
                );
            }
            AnalysisType::CGOrder => {
                Self::specify_string(
                    &mut self.analysis_type_params.cg_params.beads,
                    ui,
                    "Beads: ",
                    "Selection of lipid beads to be used in the analysis.",
                    true,
                );
            }
        });
    }

    /// Check that all required options for analysis type have been provided.
    pub(super) fn check_analysis_params_sanity(&self) -> bool {
        match self.analysis_type {
            AnalysisType::AAOrder => {
                !self.analysis_type_params.aa_params.heavy_atoms.is_empty()
                    && !self.analysis_type_params.aa_params.hydrogens.is_empty()
            }
            AnalysisType::CGOrder => !self.analysis_type_params.cg_params.beads.is_empty(),
            AnalysisType::UAOrder => {
                !self.analysis_type_params.ua_params.saturated.is_empty()
                    && !self.analysis_type_params.ua_params.unsaturated.is_empty()
            }
        }
    }
}
