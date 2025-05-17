// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Parameters for leaflet assignment.

use std::fmt::Display;

use eframe::egui::{self, RichText, Ui};
use gorder::input::{Axis, Frequency};

use crate::{common::MembraneNormal, error::ConversionError, GuiAnalysis};

/// Leaflet assignment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum LeafletClassification {
    #[default]
    None,
    Global,
    Local,
    Individual,
    Clustering,
    FromFile,
    FromNdx,
}

impl TryFrom<Option<gorder::input::LeafletClassification>> for LeafletClassification {
    type Error = ConversionError;
    fn try_from(value: Option<gorder::input::LeafletClassification>) -> Result<Self, Self::Error> {
        if let Some(leaflets) = value {
            match leaflets {
                gorder::input::LeafletClassification::Global(_) => {
                    Ok(LeafletClassification::Global)
                }
                gorder::input::LeafletClassification::Local(_) => Ok(LeafletClassification::Local),
                gorder::input::LeafletClassification::Individual(_) => {
                    Ok(LeafletClassification::Individual)
                }
                gorder::input::LeafletClassification::Clustering(_) => {
                    Ok(LeafletClassification::Clustering)
                }
                gorder::input::LeafletClassification::FromFile(_) => {
                    Ok(LeafletClassification::FromFile)
                }
                gorder::input::LeafletClassification::FromNdx(_) => {
                    Ok(LeafletClassification::FromNdx)
                }
                gorder::input::LeafletClassification::FromMap(_) => {
                    Err(ConversionError::FromMapLeaflets)
                }
            }
        } else {
            Ok(LeafletClassification::None)
        }
    }
}

impl Display for LeafletClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafletClassification::None => write!(f, ""),
            LeafletClassification::Global => write!(f, "global"),
            LeafletClassification::Local => write!(f, "local"),
            LeafletClassification::Individual => write!(f, "individual"),
            LeafletClassification::Clustering => write!(f, "clustering"),
            LeafletClassification::FromFile => write!(f, "leaflet assignment file"),
            LeafletClassification::FromNdx => write!(f, "NDX files"),
        }
    }
}

/// Parameters for leaflet assignment.
#[derive(Debug, Clone, Default)]
pub(crate) struct LeafletClassificationParams {
    global_params: LeafletGlobalParams,
    local_params: LeafletLocalParams,
    individual_params: LeafletIndividualParams,
    clustering_params: LeafletClusteringParams,
    from_file_params: LeafletFromFileParams,
    from_ndx_params: LeafletFromNdxParams,
    frequency: Frequency,
    membrane_normal: Option<MembraneNormal>,
}

fn convert_axis_option(axis: Option<Axis>) -> Option<MembraneNormal> {
    match axis {
        None => None,
        Some(x) => Some(x.into()),
    }
}

impl TryFrom<Option<gorder::input::LeafletClassification>> for LeafletClassificationParams {
    type Error = ConversionError;
    fn try_from(value: Option<gorder::input::LeafletClassification>) -> Result<Self, Self::Error> {
        if let Some(leaflets) = value {
            match leaflets {
                gorder::input::LeafletClassification::Global(params) => Ok(Self {
                    global_params: LeafletGlobalParams {
                        membrane: params.membrane().clone(),
                        heads: params.heads().clone(),
                    },
                    frequency: params.frequency(),
                    membrane_normal: convert_axis_option(params.membrane_normal().clone()),
                    ..Default::default()
                }),

                gorder::input::LeafletClassification::Local(params) => Ok(Self {
                    local_params: LeafletLocalParams {
                        membrane: params.membrane().clone(),
                        heads: params.heads().clone(),
                        radius: params.radius(),
                    },
                    frequency: params.frequency(),
                    membrane_normal: convert_axis_option(params.membrane_normal().clone()),
                    ..Default::default()
                }),

                gorder::input::LeafletClassification::Individual(params) => Ok(Self {
                    individual_params: LeafletIndividualParams {
                        heads: params.heads().clone(),
                        methyls: params.methyls().clone(),
                    },
                    frequency: params.frequency(),
                    membrane_normal: convert_axis_option(params.membrane_normal().clone()),
                    ..Default::default()
                }),

                gorder::input::LeafletClassification::Clustering(params) => Ok(Self {
                    clustering_params: LeafletClusteringParams {
                        heads: params.heads().clone(),
                    },
                    frequency: params.frequency(),
                    ..Default::default()
                }),

                gorder::input::LeafletClassification::FromFile(params) => Ok(Self {
                    from_file_params: LeafletFromFileParams {
                        file: params.file().clone(),
                    },
                    frequency: params.frequency(),
                    ..Default::default()
                }),
                gorder::input::LeafletClassification::FromNdx(params) => Ok(Self {
                    from_ndx_params: LeafletFromNdxParams {
                        ndx: params.ndx().clone(),
                        heads: params.heads().clone(),
                        upper_leaflet: params.upper_leaflet().clone(),
                        lower_leaflet: params.lower_leaflet().clone(),
                    },
                    frequency: params.frequency(),
                    ..Default::default()
                }),
                gorder::input::LeafletClassification::FromMap(_) => {
                    Err(ConversionError::FromMapLeaflets)
                }
            }
        } else {
            Ok(LeafletClassificationParams::default())
        }
    }
}

impl TryFrom<&GuiAnalysis> for Option<gorder::input::LeafletClassification> {
    type Error = ConversionError;

    fn try_from(value: &GuiAnalysis) -> Result<Self, Self::Error> {
        fn add_normal(
            method: gorder::input::LeafletClassification,
            normal: Option<MembraneNormal>,
        ) -> gorder::input::LeafletClassification {
            if let Some(normal) = normal {
                method.with_membrane_normal(normal.into())
            } else {
                method
            }
        }

        let params = &value.leaflet_classification_params;

        match value.leaflet_classification_method {
            LeafletClassification::None => Ok(None),
            LeafletClassification::Global => Ok(Some(add_normal(
                gorder::input::LeafletClassification::global(
                    &params.global_params.membrane,
                    &params.global_params.heads,
                )
                .with_frequency(params.frequency),
                params.membrane_normal,
            ))),
            LeafletClassification::Local => Ok(Some(add_normal(
                gorder::input::LeafletClassification::local(
                    &params.local_params.membrane,
                    &params.local_params.heads,
                    params.local_params.radius,
                )
                .with_frequency(params.frequency),
                params.membrane_normal,
            ))),
            LeafletClassification::Individual => Ok(Some(add_normal(
                gorder::input::LeafletClassification::individual(
                    &params.individual_params.heads,
                    &params.individual_params.methyls,
                )
                .with_frequency(params.frequency),
                params.membrane_normal,
            ))),
            LeafletClassification::Clustering => Ok(Some(
                gorder::input::LeafletClassification::clustering(&params.clustering_params.heads)
                    .with_frequency(params.frequency),
            )),
            LeafletClassification::FromFile => Ok(Some(
                gorder::input::LeafletClassification::from_file(&params.from_file_params.file)
                    .with_frequency(params.frequency),
            )),
            LeafletClassification::FromNdx => Ok(Some(
                gorder::input::LeafletClassification::from_ndx(
                    &params
                        .from_ndx_params
                        .ndx
                        .iter()
                        .map(|x| x.as_str())
                        .collect::<Vec<&str>>(),
                    &params.from_ndx_params.heads,
                    &params.from_ndx_params.upper_leaflet,
                    &params.from_ndx_params.lower_leaflet,
                )
                .with_frequency(params.frequency),
            )),
        }
    }
}

/// Frequency of the leaflet assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RawFrequency {
    Once,
    #[default]
    Every,
    EveryN,
}

/// Parameters for the global assignment method.
#[derive(Debug, Clone, Default)]
struct LeafletGlobalParams {
    membrane: String,
    heads: String,
}

impl LeafletGlobalParams {
    /// Specify the parameters for the global assignment method.
    fn specify(&mut self, ui: &mut Ui) {
        GuiAnalysis::specify_string(
            &mut self.membrane,
            ui,
            "Membrane:        ",
            "Selection of all lipid atoms forming the membrane.",
            true,
        );
        GuiAnalysis::specify_string(
            &mut self.heads,
            ui,
            "Lipid heads:     ",
            "Selection of lipid atoms representing lipid heads. One atom per molecule!",
            true,
        );
    }

    /// Check that all required parameters are provided.
    fn sanity_check(&self) -> bool {
        !self.membrane.is_empty() && !self.heads.is_empty()
    }
}

/// Parameters for the local assignment method.
#[derive(Debug, Clone)]
struct LeafletLocalParams {
    membrane: String,
    heads: String,
    radius: f32,
}

impl LeafletLocalParams {
    /// Specify the parameters for the local assignment method.
    fn specify(&mut self, ui: &mut Ui) {
        GuiAnalysis::specify_string(
            &mut self.membrane,
            ui,
            "Membrane:       ",
            "Selection of all lipid atoms forming the membrane.",
            true,
        );

        GuiAnalysis::specify_string(
            &mut self.heads,
            ui,
            "Lipid heads:    ",
            "Selection of lipid atoms representing lipid heads. One atom per molecule!",
            true,
        );

        ui.horizontal(|ui| {
            let label = GuiAnalysis::label_with_hint(
                ui,
                "Radius:         ",
                "Radius of the cylinder for the calculation of local membrane center.",
            );

            ui.add(
                egui::DragValue::new(&mut self.radius)
                    .speed(0.025)
                    .range(0.0..=f32::MAX)
                    .suffix(" nm"),
            )
            .labelled_by(label.id);

            if self.radius == 0.0 {
                ui.label(
                    RichText::new("❗")
                        .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)),
                );
            }
        });
    }

    /// Check that all required parameters are provided.
    fn sanity_check(&self) -> bool {
        !self.membrane.is_empty() && !self.heads.is_empty() && self.radius > 0.0
    }
}

impl Default for LeafletLocalParams {
    fn default() -> Self {
        LeafletLocalParams {
            membrane: String::from(""),
            heads: String::from(""),
            radius: 2.5,
        }
    }
}

/// Parameters for the individual assignment method.
#[derive(Debug, Clone, Default)]
struct LeafletIndividualParams {
    heads: String,
    methyls: String,
}

impl LeafletIndividualParams {
    /// Specify the parameters for the individual assignment method.
    fn specify(&mut self, ui: &mut Ui) {
        GuiAnalysis::specify_string(
            &mut self.heads,
            ui,
            "Lipid heads:     ",
            "Selection of lipid atoms representing lipid heads. One atom per molecule!",
            true,
        );

        GuiAnalysis::specify_string(
            &mut self.methyls,
            ui,
            "Lipid methyls:   ",
            "Selection of lipid atoms representing ends of lipid tails.",
            true,
        );
    }

    /// Check that all required parameters are provided.
    fn sanity_check(&self) -> bool {
        !self.heads.is_empty() && !self.methyls.is_empty()
    }
}

/// Parameters for the clustering assignment method.
#[derive(Debug, Clone, Default)]
struct LeafletClusteringParams {
    heads: String,
}

impl LeafletClusteringParams {
    /// Specify the parameters for the clustering assignment method.
    fn specify(&mut self, ui: &mut Ui) {
        GuiAnalysis::specify_string(
            &mut self.heads,
            ui,
            "Lipid heads: ",
            "Selection of lipid atoms representing lipid heads. One atom per molecule!",
            true,
        );
    }

    /// Check that all required parameters are provided.
    fn sanity_check(&self) -> bool {
        !self.heads.is_empty()
    }
}

/// Parameters for the "from file" assignment method.
#[derive(Debug, Clone, Default)]
struct LeafletFromFileParams {
    file: String,
}

impl LeafletFromFileParams {
    /// Specify the parameters for the "from file" assignment method.
    fn specify(&mut self, ui: &mut Ui) {
        GuiAnalysis::specify_input_file(
            &mut self.file,
            ui,
            "Input file:  ",
            "Path to a leaflet assignment file.",
            true,
        );
    }

    /// Check that all required parameters are provided.
    fn sanity_check(&self) -> bool {
        !self.file.is_empty()
    }
}

/// Parameters for the "from NDX" assignment method.
#[derive(Debug, Clone, Default)]
struct LeafletFromNdxParams {
    heads: String,
    ndx: Vec<String>,
    upper_leaflet: String,
    lower_leaflet: String,
}

impl LeafletFromNdxParams {
    /// Specify the parameters for the "from NDX" assignment method.
    fn specify(&mut self, ui: &mut Ui) {
        GuiAnalysis::specify_multiple_input_files(
            &mut self.ndx,
            ui,
            "NDX files: ",
            "Path to NDX files specifying the leaflets.",
            true,
        );
        GuiAnalysis::specify_string(
            &mut self.heads,
            ui,
            "Lipid heads:   ",
            "Selection of lipid atoms representing lipid heads. One atom per molecule!",
            true,
        );
        GuiAnalysis::specify_string(
            &mut self.upper_leaflet,
            ui,
            "Upper leaflet: ",
            "Name of the NDX group containing atoms of the upper membrane leaflet.",
            true,
        );
        GuiAnalysis::specify_string(
            &mut self.lower_leaflet,
            ui,
            "Lower leaflet: ",
            "Name of the NDX group containing atoms of the lower membrane leaflet.",
            true,
        );
    }

    /// Check that all required parameters are provided.
    fn sanity_check(&self) -> bool {
        !self.heads.is_empty()
            && !self.ndx.iter().any(|file| file.is_empty())
            && !self.upper_leaflet.is_empty()
            && !self.lower_leaflet.is_empty()
    }
}

impl GuiAnalysis {
    /// Specify the method for leaflet assignment and the required parameters.
    pub(super) fn specify_leaflet_classification(&mut self, ui: &mut Ui) {
        Self::collapsing_with_warning(
            ui,
            "Leaflet assignment",
            false,
            self.check_leaflets_sanity(),
            |ui| {
                ui.horizontal(|ui| {
                    Self::label_with_hint(
                        ui,
                        "Assignment method: ",
                        "Method to use for assigning lipids into membrane leaflets.",
                    );

                    egui::ComboBox::from_label("")
                        .selected_text(format!("{}", self.leaflet_classification_method))
                        .show_ui(ui, |ui| {
                            for variant in [
                                LeafletClassification::None,
                                LeafletClassification::Global,
                                LeafletClassification::Local,
                                LeafletClassification::Individual,
                                LeafletClassification::Clustering,
                                LeafletClassification::FromFile,
                                LeafletClassification::FromNdx,
                            ] {
                                ui.selectable_value(
                                    &mut self.leaflet_classification_method,
                                    variant.clone(),
                                    format!("{}", variant),
                                );
                            }
                        });

                    if let LeafletClassification::None = self.leaflet_classification_method {
                        ui.label(
                            RichText::new("no leaflet assignment")
                                .font(egui::FontId::proportional(10.0)),
                        );
                    }
                    ui.end_row();
                });

                ui.vertical(|ui| match self.leaflet_classification_method {
                    LeafletClassification::None => (),
                    LeafletClassification::Global => {
                        self.leaflet_classification_params.global_params.specify(ui);

                        Self::specify_frequency(
                            &mut self.leaflet_classification_params.frequency,
                            ui,
                            "Frequency:       ",
                        );
                        self.specify_leaflet_membrane_normal(ui, "Membrane normal: ");
                    }
                    LeafletClassification::Local => {
                        self.leaflet_classification_params.local_params.specify(ui);

                        Self::specify_frequency(
                            &mut self.leaflet_classification_params.frequency,
                            ui,
                            "Frequency:      ",
                        );

                        self.specify_leaflet_membrane_normal(ui, "Membrane normal:");
                    }
                    LeafletClassification::Individual => {
                        self.leaflet_classification_params
                            .individual_params
                            .specify(ui);

                        Self::specify_frequency(
                            &mut self.leaflet_classification_params.frequency,
                            ui,
                            "Frequency:       ",
                        );

                        self.specify_leaflet_membrane_normal(ui, "Membrane normal: ");
                    }
                    LeafletClassification::Clustering => {
                        self.leaflet_classification_params
                            .clustering_params
                            .specify(ui);

                        Self::specify_frequency(
                            &mut self.leaflet_classification_params.frequency,
                            ui,
                            "Frequency:   ",
                        );
                    }
                    LeafletClassification::FromFile => {
                        self.leaflet_classification_params
                            .from_file_params
                            .specify(ui);

                        Self::specify_frequency(
                            &mut self.leaflet_classification_params.frequency,
                            ui,
                            "Frequency:   ",
                        );
                    }
                    LeafletClassification::FromNdx => {
                        self.leaflet_classification_params
                            .from_ndx_params
                            .specify(ui);

                        Self::specify_frequency(
                            &mut self.leaflet_classification_params.frequency,
                            ui,
                            "Frequency:     ",
                        );
                    }
                });
            },
        );
    }

    /// Specify the frequency of leaflet assignment.
    fn specify_frequency(frequency: &mut Frequency, ui: &mut Ui, label: &str) {
        ui.horizontal(|ui| {
            Self::label_with_hint(ui, label, "Frequency of the leaflet assignment.");

            let (mut raw_frequency, mut n) = match frequency {
                Frequency::Once => (RawFrequency::Once, 0),
                Frequency::Every(n) if n.get() == 1 => (RawFrequency::Every, 1),
                Frequency::Every(n) => (RawFrequency::EveryN, n.get()),
            };

            ui.radio_value(&mut raw_frequency, RawFrequency::Once, "once");
            ui.radio_value(&mut raw_frequency, RawFrequency::Every, "every frame");
            ui.radio_value(&mut raw_frequency, RawFrequency::EveryN, "every Nth frame");

            match raw_frequency {
                RawFrequency::Once => *frequency = Frequency::once(),
                RawFrequency::Every => *frequency = Frequency::every(1).unwrap(),
                RawFrequency::EveryN => {
                    if n < 2 {
                        n = 2;
                    }

                    ui.add(
                        egui::DragValue::new(&mut n)
                            .update_while_editing(false)
                            .range(1..=usize::MAX)
                            .speed(1)
                            .prefix("N = "),
                    );

                    *frequency = Frequency::every(n).unwrap();
                }
            }
        });
    }

    /// Specify the membrane normal for leaflet assignment.
    fn specify_leaflet_membrane_normal(&mut self, ui: &mut Ui, label: &str) {
        // if membrane normal is not explicitly provided for the leaflet assignment, use the global membrane normal
        let raw_normal = if let Some(x) = self.leaflet_classification_params.membrane_normal {
            x
        } else {
            self.membrane_normal
        };

        ui.horizontal(|ui| {
            Self::label_with_hint(
                ui,
                label,
                "Membrane normal used for the leaflet classification. Can be decoupled from the global membrane normal."
            );

            // if any of these buttons is clicked, membrane normal for leaflet assignment 
            // is permanently decoupled from the global membrane normal
            if ui.add(egui::RadioButton::new(raw_normal == MembraneNormal::X, "x")).clicked() {
                self.leaflet_classification_params.membrane_normal = Some(MembraneNormal::X);
            }

            if ui.add(egui::RadioButton::new(raw_normal == MembraneNormal::Y, "y")).clicked() {
                self.leaflet_classification_params.membrane_normal = Some(MembraneNormal::Y);
            }

            if ui.add(egui::RadioButton::new(raw_normal == MembraneNormal::Z, "z")).clicked() {
                self.leaflet_classification_params.membrane_normal = Some(MembraneNormal::Z);
            }

            // signal to the user that membrane normal must be explicitly set
            if raw_normal == MembraneNormal::Dynamic || raw_normal == MembraneNormal::FromFile {
                ui.label(RichText::new("❗").color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)));
            }
        });
    }

    /// Check that all required options for leaflet assignment have been provided.
    pub(super) fn check_leaflets_sanity(&self) -> bool {
        (match self.leaflet_classification_method {
            LeafletClassification::None => true,
            LeafletClassification::Global => self
                .leaflet_classification_params
                .global_params
                .sanity_check(),

            LeafletClassification::Local => self
                .leaflet_classification_params
                .local_params
                .sanity_check(),
            LeafletClassification::Individual => self
                .leaflet_classification_params
                .individual_params
                .sanity_check(),
            LeafletClassification::Clustering => self
                .leaflet_classification_params
                .clustering_params
                .sanity_check(),
            LeafletClassification::FromFile => self
                .leaflet_classification_params
                .from_file_params
                .sanity_check(),
            LeafletClassification::FromNdx => self
                .leaflet_classification_params
                .from_ndx_params
                .sanity_check(),
        }) && (match self.leaflet_classification_method {
            LeafletClassification::Global
            | LeafletClassification::Local
            | LeafletClassification::Individual => {
                self.membrane_normal != MembraneNormal::Dynamic
                    || self.leaflet_classification_params.membrane_normal.is_some()
            }
            _ => true,
        })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn convert_classification_type() {
        assert_eq!(
            LeafletClassification::try_from(None).unwrap(),
            LeafletClassification::None
        );

        assert_eq!(
            LeafletClassification::try_from(Some(gorder::input::LeafletClassification::global(
                "@membrane",
                "name P"
            )))
            .unwrap(),
            LeafletClassification::Global
        );

        assert_eq!(
            LeafletClassification::try_from(Some(gorder::input::LeafletClassification::local(
                "@membrane",
                "name P",
                2.5
            )))
            .unwrap(),
            LeafletClassification::Local
        );

        assert_eq!(
            LeafletClassification::try_from(Some(
                gorder::input::LeafletClassification::individual("name P", "name C218 C316",)
            ))
            .unwrap(),
            LeafletClassification::Individual
        );

        assert_eq!(
            LeafletClassification::try_from(Some(
                gorder::input::LeafletClassification::clustering("name P")
            ))
            .unwrap(),
            LeafletClassification::Clustering
        );

        assert_eq!(
            LeafletClassification::try_from(Some(gorder::input::LeafletClassification::from_file(
                "leaflets.yaml"
            )))
            .unwrap(),
            LeafletClassification::FromFile
        );

        assert_eq!(
            LeafletClassification::try_from(Some(gorder::input::LeafletClassification::from_ndx(
                &["leaflets1.ndx", "leaflets2.ndx", "leaflets3.ndx"],
                "name P",
                "UpperLeaflet",
                "LowerLeaflet"
            )))
            .unwrap(),
            LeafletClassification::FromNdx
        );
    }

    #[test]
    fn convert_classification_params() {
        let params = LeafletClassificationParams::try_from(Some(
            gorder::input::LeafletClassification::global("@membrane", "name P"),
        ))
        .unwrap();

        assert_eq!(params.global_params.membrane, String::from("@membrane"));
        assert_eq!(params.global_params.heads, String::from("name P"));
        assert_eq!(params.frequency, Frequency::every(1).unwrap());
        assert!(params.membrane_normal.is_none());

        let params = LeafletClassificationParams::try_from(Some(
            gorder::input::LeafletClassification::local("@membrane", "name P", 2.5)
                .with_frequency(Frequency::Once),
        ))
        .unwrap();

        assert_eq!(params.local_params.membrane, String::from("@membrane"));
        assert_eq!(params.local_params.heads, String::from("name P"));
        assert_relative_eq!(params.local_params.radius, 2.5);
        assert_eq!(params.frequency, Frequency::Once);
        assert!(params.membrane_normal.is_none());

        let params = LeafletClassificationParams::try_from(Some(
            gorder::input::LeafletClassification::individual("name P", "name C218 C316")
                .with_membrane_normal(Axis::Y),
        ))
        .unwrap();

        assert_eq!(params.individual_params.heads, String::from("name P"));
        assert_eq!(
            params.individual_params.methyls,
            String::from("name C218 C316")
        );
        assert_eq!(params.frequency, Frequency::every(1).unwrap());
        assert_eq!(params.membrane_normal, Some(MembraneNormal::Y));

        let params = LeafletClassificationParams::try_from(Some(
            gorder::input::LeafletClassification::clustering("name P")
                .with_frequency(Frequency::once()),
        ))
        .unwrap();

        assert_eq!(params.clustering_params.heads, String::from("name P"));
        assert_eq!(params.frequency, Frequency::once());

        let params = LeafletClassificationParams::try_from(Some(
            gorder::input::LeafletClassification::from_file("leaflets.yaml"),
        ))
        .unwrap();

        assert_eq!(params.from_file_params.file, String::from("leaflets.yaml"));
        assert_eq!(params.frequency, Frequency::every(1).unwrap());

        let params = LeafletClassificationParams::try_from(Some(
            gorder::input::LeafletClassification::from_ndx(
                &["leaflets1.ndx", "leaflets2.ndx", "leaflets3.ndx"],
                "name P",
                "UpperLeaflet",
                "LowerLeaflet",
            )
            .with_frequency(Frequency::every(5).unwrap()),
        ))
        .unwrap();

        assert_eq!(
            params.from_ndx_params.ndx,
            vec![
                String::from("leaflets1.ndx"),
                String::from("leaflets2.ndx"),
                String::from("leaflets3.ndx")
            ]
        );
        assert_eq!(params.from_ndx_params.heads, String::from("name P"));
        assert_eq!(
            params.from_ndx_params.upper_leaflet,
            String::from("UpperLeaflet")
        );
        assert_eq!(
            params.from_ndx_params.lower_leaflet,
            String::from("LowerLeaflet")
        );
        assert_eq!(params.frequency, Frequency::every(5).unwrap());
    }
}
