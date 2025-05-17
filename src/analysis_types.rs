// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Analysis types and their parameters.

use eframe::egui::Ui;

use crate::GuiAnalysis;

/// Type of analysis to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum AnalysisType {
    #[default]
    AAOrder,
    UAOrder,
    CGOrder,
}

impl From<gorder::input::AnalysisType> for AnalysisType {
    fn from(value: gorder::input::AnalysisType) -> Self {
        match value {
            gorder::input::AnalysisType::AAOrder { .. } => AnalysisType::AAOrder,
            gorder::input::AnalysisType::CGOrder { .. } => AnalysisType::CGOrder,
            gorder::input::AnalysisType::UAOrder { .. } => AnalysisType::UAOrder,
        }
    }
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

impl From<gorder::input::AnalysisType> for AnalysisTypeParams {
    fn from(value: gorder::input::AnalysisType) -> Self {
        match value {
            gorder::input::AnalysisType::AAOrder {
                heavy_atoms,
                hydrogens,
            } => Self {
                aa_params: AAParams {
                    heavy_atoms,
                    hydrogens,
                },
                ..Default::default()
            },
            gorder::input::AnalysisType::CGOrder { beads } => Self {
                cg_params: CGParams { beads },
                ..Default::default()
            },
            gorder::input::AnalysisType::UAOrder {
                saturated,
                unsaturated,
                ignore,
            } => Self {
                ua_params: UAParams {
                    saturated: saturated.unwrap_or(String::new()),
                    unsaturated: unsaturated.unwrap_or(String::new()),
                    ignore: ignore.unwrap_or(String::new()),
                },
                ..Default::default()
            },
        }
    }
}

impl From<&GuiAnalysis> for gorder::input::AnalysisType {
    fn from(value: &GuiAnalysis) -> Self {
        match value.analysis_type {
            AnalysisType::AAOrder => gorder::input::AnalysisType::aaorder(
                &value.analysis_type_params.aa_params.heavy_atoms,
                &value.analysis_type_params.aa_params.hydrogens,
            ),
            AnalysisType::CGOrder => {
                gorder::input::AnalysisType::cgorder(&value.analysis_type_params.cg_params.beads)
            }
            AnalysisType::UAOrder => {
                let uaparams = &value.analysis_type_params.ua_params;
                let saturated = match uaparams.saturated.is_empty() {
                    true => None,
                    false => Some(&uaparams.saturated),
                };

                let unsaturated = match uaparams.unsaturated.is_empty() {
                    true => None,
                    false => Some(&uaparams.unsaturated),
                };

                let ignore = match uaparams.ignore.is_empty() {
                    true => None,
                    false => Some(&uaparams.ignore),
                };

                gorder::input::AnalysisType::uaorder(
                    saturated.map(|x| x.as_str()),
                    unsaturated.map(|x| x.as_str()),
                    ignore.map(|x| x.as_str()),
                )
            }
        }
    }
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
                    false,
                );
                Self::specify_string(
                    &mut self.analysis_type_params.ua_params.unsaturated,
                    ui,
                    "Unsaturated carbons: ",
                    "Selection of unsaturated carbons to be used in the analysis.",
                    false,
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
            AnalysisType::UAOrder => true, // no compulsory parameters
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_analysis_type() {
        assert_eq!(
            AnalysisType::from(gorder::input::AnalysisType::aaorder(
                "element name carbon",
                "element name hydrogen"
            )),
            AnalysisType::AAOrder,
        );

        assert_eq!(
            AnalysisType::from(gorder::input::AnalysisType::cgorder("@membrane")),
            AnalysisType::CGOrder,
        );

        assert_eq!(
            AnalysisType::from(gorder::input::AnalysisType::uaorder(
                Some("name C211 C212"),
                Some("name C29 C210"),
                None
            )),
            AnalysisType::UAOrder,
        )
    }

    #[test]
    fn convert_analysis_params() {
        let params = AnalysisTypeParams::from(gorder::input::AnalysisType::aaorder(
            "element name carbon",
            "element name hydrogen",
        ));

        assert_eq!(
            params.aa_params.heavy_atoms,
            String::from("element name carbon")
        );
        assert_eq!(
            params.aa_params.hydrogens,
            String::from("element name hydrogen")
        );

        let params = AnalysisTypeParams::from(gorder::input::AnalysisType::cgorder("@membrane"));
        assert_eq!(params.cg_params.beads, String::from("@membrane"));

        let params = AnalysisTypeParams::from(gorder::input::AnalysisType::uaorder(
            Some("name C211 C212"),
            Some("name C29 C210"),
            Some("element symbol H"),
        ));

        assert_eq!(params.ua_params.saturated, String::from("name C211 C212"));
        assert_eq!(params.ua_params.unsaturated, String::from("name C29 C210"));
        assert_eq!(params.ua_params.ignore, String::from("element symbol H"));
    }
}
