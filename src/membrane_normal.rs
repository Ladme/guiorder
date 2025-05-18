// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Parameters related to membrane normal.

use eframe::egui::{self, RichText, Ui};
use gorder::input::{Axis, DynamicNormal};

use crate::{common::MembraneNormal, error::ConversionError, GuiAnalysis};

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

impl TryFrom<gorder::input::MembraneNormal> for DynamicNormalParams {
    type Error = ConversionError;
    fn try_from(value: gorder::input::MembraneNormal) -> Result<Self, Self::Error> {
        match value {
            gorder::input::MembraneNormal::Dynamic(dynamic) => Ok(Self {
                heads: dynamic.heads().clone(),
                radius: dynamic.radius(),
            }),
            gorder::input::MembraneNormal::FromMap(_) => Err(ConversionError::FromMapNormals),
            gorder::input::MembraneNormal::Static(_) => Ok(Self::default()),
            gorder::input::MembraneNormal::FromFile(_) => Ok(Self::default()),
        }
    }
}

impl TryFrom<&GuiAnalysis> for gorder::input::MembraneNormal {
    type Error = ConversionError;

    fn try_from(value: &GuiAnalysis) -> Result<Self, Self::Error> {
        match value.membrane_normal {
            MembraneNormal::X => Ok(gorder::input::MembraneNormal::Static(Axis::X)),
            MembraneNormal::Y => Ok(gorder::input::MembraneNormal::Static(Axis::Y)),
            MembraneNormal::Z => Ok(gorder::input::MembraneNormal::Static(Axis::Z)),
            MembraneNormal::FromFile => Ok(gorder::input::MembraneNormal::FromFile(
                value.from_file_normals.clone(),
            )),
            MembraneNormal::Dynamic => Ok(gorder::input::MembraneNormal::Dynamic(
                DynamicNormal::new(
                    &value.dynamic_normal_params.heads,
                    value.dynamic_normal_params.radius,
                )
                .map_err(|e| ConversionError::InvalidMembraneNormal(e.to_string()))?,
            )),
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
                    ui.radio_value(
                        &mut self.membrane_normal,
                        MembraneNormal::FromFile,
                        "from file",
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
                                .speed(0.025)
                                .range(0.0..=f32::MAX)
                                .suffix(" nm"),
                            )
                            .labelled_by(label.id);

                            if self.dynamic_normal_params.radius == 0.0 {
                                ui.label(
                                    RichText::new("❗")
                                        .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)),
                                );
                            }
                        });
                    });
                } else if self.membrane_normal == MembraneNormal::FromFile {
                    GuiAnalysis::specify_input_file(
                        &mut self.from_file_normals,
                        ui,
                        "Normals file: ",
                        "Path to a file specifying the membrane normals to use for individual lipid molecules.",
                        true
                    );
                }
            },
        );
    }

    /// Check that all required options for membrane normal specification have been provided.
    pub(super) fn check_membrane_normal_sanity(&self) -> bool {
        match self.membrane_normal {
            MembraneNormal::Dynamic => {
                !self.dynamic_normal_params.heads.is_empty()
                    && self.dynamic_normal_params.radius > 0.0
            }
            MembraneNormal::FromFile => !self.from_file_normals.is_empty(),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use gorder::input::DynamicNormal;

    use super::*;

    #[test]
    fn gorder_to_guiorder_dynamic_normal_params() {
        let params = DynamicNormalParams::try_from(gorder::input::MembraneNormal::Dynamic(
            DynamicNormal::new("name P", 2.5).unwrap(),
        ))
        .unwrap();

        assert_eq!(params.heads, String::from("name P"));
        assert_relative_eq!(params.radius, 2.5);
    }

    #[test]
    fn guiorder_to_gorder_membrane_normal_z() {
        let params = GuiAnalysis {
            membrane_normal: MembraneNormal::Z,
            ..Default::default()
        };

        match gorder::input::MembraneNormal::try_from(&params).unwrap() {
            gorder::input::MembraneNormal::Static(axis) => {
                assert!(matches!(axis, Axis::Z));
            }
            _ => panic!("Invalid membrane normal."),
        }
    }

    #[test]
    fn guiorder_to_gorder_membrane_normal_y() {
        let params = GuiAnalysis {
            membrane_normal: MembraneNormal::Y,
            ..Default::default()
        };

        match gorder::input::MembraneNormal::try_from(&params).unwrap() {
            gorder::input::MembraneNormal::Static(axis) => {
                assert!(matches!(axis, Axis::Y));
            }
            _ => panic!("Invalid membrane normal."),
        }
    }

    #[test]
    fn guiorder_to_gorder_membrane_normal_x() {
        let params = GuiAnalysis {
            membrane_normal: MembraneNormal::X,
            ..Default::default()
        };

        match gorder::input::MembraneNormal::try_from(&params).unwrap() {
            gorder::input::MembraneNormal::Static(axis) => {
                assert!(matches!(axis, Axis::X));
            }
            _ => panic!("Invalid membrane normal."),
        }
    }

    #[test]
    fn guiorder_to_gorder_membrane_normal_dynamic() {
        let params = GuiAnalysis {
            membrane_normal: MembraneNormal::Dynamic,
            dynamic_normal_params: DynamicNormalParams {
                heads: String::from("name P"),
                radius: 1.75,
            },
            ..Default::default()
        };

        match gorder::input::MembraneNormal::try_from(&params).unwrap() {
            gorder::input::MembraneNormal::Dynamic(converted) => {
                assert_eq!(converted.heads(), &String::from("name P"));
                assert_relative_eq!(converted.radius(), 1.75);
            }
            _ => panic!("Invalid membrane normal."),
        }
    }

    #[test]
    fn guiorder_to_gorder_membrane_normal_from_file() {
        let params = GuiAnalysis {
            membrane_normal: MembraneNormal::FromFile,
            from_file_normals: String::from("normals.yaml"),
            ..Default::default()
        };

        match gorder::input::MembraneNormal::try_from(&params).unwrap() {
            gorder::input::MembraneNormal::FromFile(file) => {
                assert_eq!(file, String::from("normals.yaml"));
            }
            _ => panic!("Invalid membrane normal."),
        }
    }
}
