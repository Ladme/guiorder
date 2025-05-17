// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Common structures and methods.

use eframe::egui::{self, CollapsingResponse, CursorIcon, Response, RichText, Ui};
use gorder::input::Axis;

use crate::{
    analysis_types::{AnalysisType, AnalysisTypeParams},
    error::ConversionError,
    estimate_error::EstimateErrorParams,
    frame_selection::FrameSelectionParams,
    geometry::{GeomSelection, GeomSelectionParams},
    membrane_normal::DynamicNormalParams,
    ordermaps::OrderMapsParams,
    other_options::OtherParams,
    LeafletClassification, LeafletClassificationParams, OutputFiles,
};

/// Main structure handling the drawing of the GUI and the collection of input.
#[derive(Debug, Clone, Default)]
pub(crate) struct GuiAnalysis {
    pub structure: String,
    pub trajectory: Vec<String>,
    pub ndx: String,
    pub bonds: String,
    pub analysis_type: AnalysisType,
    pub analysis_type_params: AnalysisTypeParams,
    pub output: OutputFiles,
    pub leaflet_classification_method: LeafletClassification,
    pub leaflet_classification_params: LeafletClassificationParams,
    pub membrane_normal: MembraneNormal,
    pub dynamic_normal_params: DynamicNormalParams,
    pub from_file_normals: String,
    pub ordermaps_params: OrderMapsParams,
    pub estimate_error_params: EstimateErrorParams,
    pub frame_selection_params: FrameSelectionParams,
    pub other_params: OtherParams,
    pub geom_selection: GeomSelection,
    pub geom_selection_params: GeomSelectionParams,
}

/// Direction of the membrane nornal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum MembraneNormal {
    X,
    Y,
    #[default]
    Z,
    Dynamic,
    FromFile,
}

impl From<Axis> for MembraneNormal {
    fn from(value: Axis) -> Self {
        match value {
            Axis::X => MembraneNormal::X,
            Axis::Y => MembraneNormal::Y,
            Axis::Z => MembraneNormal::Z,
        }
    }
}

impl TryFrom<gorder::input::MembraneNormal> for MembraneNormal {
    type Error = ConversionError;
    fn try_from(value: gorder::input::MembraneNormal) -> Result<Self, Self::Error> {
        match value {
            gorder::input::MembraneNormal::Static(axis) => Ok(axis.into()),
            gorder::input::MembraneNormal::Dynamic(_) => Ok(Self::Dynamic),
            gorder::input::MembraneNormal::FromFile(_) => Ok(Self::FromFile),
            gorder::input::MembraneNormal::FromMap(_) => Err(ConversionError::FromMapNormals),
        }
    }
}

impl GuiAnalysis {
    /// Specify input file either using a text input or by interactive selection.
    pub(crate) fn specify_input_file(
        target: &mut String,
        ui: &mut Ui,
        label: &str,
        hint: &str,
        required: bool,
    ) {
        ui.horizontal(|ui| {
            Self::label_with_hint(ui, label, hint);
            Self::text_field(target, ui, required);

            if ui
                .button("📁")
                .on_hover_ui(|ui| {
                    ui.label("Select the file interactively.");
                })
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().set_directory(".").pick_file() {
                    *target = path.display().to_string();
                }
            }
        });
    }

    /// Specify multiple input files either using a text input or by interactive selection.
    pub(crate) fn specify_multiple_input_files(
        target: &mut Vec<String>,
        ui: &mut Ui,
        label: &str,
        hint: &str,
        required: bool,
    ) {
        if target.is_empty() {
            target.push(String::new());
        }

        if target.len() == 1 {
            ui.horizontal(|ui| {
                Self::label_with_hint(ui, label, hint);

                if target.is_empty() {
                    target.push(String::new());
                }

                Self::text_field(&mut target[0], ui, required);
                Self::add_file_button(target, ui, !target.iter().any(|x| x.is_empty()));

                if ui
                    .button("📁")
                    .on_hover_ui(|ui| {
                        ui.label("Select the file(s) interactively.");
                    })
                    .clicked()
                {
                    if let Some(paths) = rfd::FileDialog::new().set_directory(".").pick_files() {
                        target.clear();
                        for file in paths {
                            target.push(file.display().to_string())
                        }
                    }
                }
            });
        } else {
            ui.horizontal(|ui| {
                Self::collapsing_with_warning(ui, label, true, !target.iter().any(|file| file.is_empty()), |ui| {
                    let mut index_to_remove = None;
                    let mut move_up = None;
                    let mut move_down = None;
                    let last_index = target.len() - 1;
                    let no_empty = !target.iter().any(|x| x.is_empty());
                    for (i, item) in target.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            Self::text_field(item, ui, required);

                            if ui.add_enabled(i != 0, egui::Button::new("🔼"))
                                .on_hover_ui(|ui| {ui.label("Move up in the list.");})
                                .on_disabled_hover_ui(|ui| {ui.label("Cannot move further up.");})
                                .clicked()
                            {
                                move_up = Some(i);
                            }

                            if ui.add_enabled(i != last_index, egui::Button::new("🔽"))
                                .on_hover_ui(|ui| {ui.label("Move down in the list.");})
                                .on_disabled_hover_ui(|ui| {ui.label("Cannot move further down.");})
                                .clicked()
                            {
                                move_down = Some(i);
                            }

                            if ui
                                .button("➖")
                                .on_hover_ui(|ui| {
                                    ui.label("Unselect this file.");
                                })
                                .clicked()
                            {
                                index_to_remove = Some(i);
                            }
                        });
                    }

                    if let Some(i) = index_to_remove {
                        target.remove(i);
                    }

                    if let Some(i) = move_up {
                        target.swap(i, i - 1);
                    }

                    if let Some(i) = move_down {
                        target.swap(i, i + 1);
                    }


                    ui.horizontal(|ui| {
                        ui.add_space(140.0);
                        Self::add_file_button(target, ui, no_empty);
                    });
                });

                if ui
                    .button("📁")
                    .on_hover_ui(|ui| {
                        ui.label("Select the files interactively. This will replace all already selected files!");
                    })
                    .clicked()
                {
                    if let Some(paths) = rfd::FileDialog::new().set_directory(".").pick_files() {
                        target.clear();
                        for file in paths {
                            target.push(file.display().to_string())
                        }
                    }
                }
                });
        }
    }

    /// Create a button that adds additional file to the list of selected files.
    fn add_file_button(target: &mut Vec<String>, ui: &mut Ui, not_empty: bool) {
        if ui
            .add_enabled(not_empty, egui::Button::new("➕"))
            .on_hover_ui(|ui| {
                ui.label("Select another file.");
            })
            .on_disabled_hover_ui(|ui| {
                ui.label("Cannot select another file because a previous filename is missing.");
            })
            .clicked()
        {
            target.push(String::new());
        }
    }

    /// Specify an output file either by using a text or by interactive selection.
    pub(crate) fn specify_output_file(
        target: &mut String,
        ui: &mut Ui,
        label: &str,
        hint: &str,
        required: bool,
    ) {
        ui.horizontal(|ui| {
            Self::label_with_hint(ui, label, hint);
            Self::text_field(target, ui, required);

            if ui
                .button("📁")
                .on_hover_ui(|ui| {
                    ui.label("Specify the file interactively.");
                })
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().set_directory(".").save_file() {
                    *target = path.display().to_string();
                }
            }
        });
    }

    /// Print a label with a hint that appears on hover.
    pub(crate) fn label_with_hint(ui: &mut Ui, label: &str, hint: &str) -> Response {
        ui.label(RichText::new(label).font(egui::FontId::monospace(12.0)))
            .on_hover_ui(|ui| {
                ui.label(hint);
            })
            .on_hover_cursor(CursorIcon::Help)
    }

    /// Print label and an associated text field.
    pub(crate) fn specify_string(
        target: &mut String,
        ui: &mut Ui,
        label: &str,
        hint: &str,
        required: bool,
    ) {
        ui.horizontal(|ui| {
            Self::label_with_hint(ui, label, hint);
            Self::text_field(target, ui, required);
        });
    }

    /// Create a text field. 'Required' text fields will be colored red if empty.
    fn text_field(target: &mut String, ui: &mut Ui, required: bool) {
        if required && target.is_empty() {
            ui.add(
                egui::TextEdit::singleline(target)
                    .background_color(egui::Color32::from_rgba_premultiplied(50, 0, 0, 50)),
            );
        } else {
            ui.add(egui::TextEdit::singleline(target));
        }
    }

    /// Collapsing environment which heading gets colored red if there is an error inside the environment.
    pub(crate) fn collapsing_with_warning<R>(
        ui: &mut Ui,
        heading: &str,
        open: bool,
        sanity_check: bool,
        contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        let heading = if sanity_check {
            RichText::new(heading).font(egui::FontId::monospace(12.0))
        } else {
            RichText::new(heading)
                .font(egui::FontId::monospace(12.0))
                .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100))
        };

        egui::CollapsingHeader::new(heading)
            .default_open(open)
            .show(ui, contents)
    }

    /// Radio button that can be toggled off, deselecting all options.
    #[allow(unused)]
    pub(crate) fn toggle_radio<T: PartialEq + Clone>(
        ui: &mut Ui,
        current: &mut Option<T>,
        value: T,
        text: impl Into<egui::WidgetText>,
    ) -> egui::Response {
        let is_selected = *current == Some(value.clone());
        let response = ui.radio(is_selected, text);

        if response.clicked() {
            *current = if is_selected { None } else { Some(value) };
        }

        response
    }

    /// A button that can be disabled showing different hints in enabled and multiple disabled states.
    pub(crate) fn smart_button(
        ui: &mut Ui,
        enabled: bool,
        running: bool,
        text: &str,
        enabled_hint: &str,
        disabled_hint: &str,
        running_hint: &str,
    ) -> egui::Response {
        if !running {
            ui.add_enabled(enabled, egui::Button::new(text))
                .on_hover_ui(|ui| {
                    ui.label(enabled_hint);
                })
                .on_disabled_hover_ui(|ui| {
                    ui.label(disabled_hint);
                })
        } else {
            ui.add_enabled(false, egui::Button::new(text))
                .on_disabled_hover_ui(|ui| {
                    ui.label(running_hint);
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use gorder::input::DynamicNormal;

    use super::*;

    #[test]
    fn convert_axis() {
        assert_eq!(
            MembraneNormal::from(gorder::input::Axis::X),
            MembraneNormal::X
        );

        assert_eq!(
            MembraneNormal::from(gorder::input::Axis::Y),
            MembraneNormal::Y
        );

        assert_eq!(
            MembraneNormal::from(gorder::input::Axis::Z),
            MembraneNormal::Z
        );
    }

    #[test]
    fn convert_normal() {
        assert_eq!(
            MembraneNormal::try_from(gorder::input::MembraneNormal::Static(
                gorder::input::Axis::Z
            ))
            .unwrap(),
            MembraneNormal::Z
        );

        assert_eq!(
            MembraneNormal::try_from(gorder::input::MembraneNormal::Dynamic(
                DynamicNormal::new("name P", 2.0).unwrap()
            ))
            .unwrap(),
            MembraneNormal::Dynamic
        );

        assert_eq!(
            MembraneNormal::try_from(gorder::input::MembraneNormal::FromFile(String::from(
                "normals.yaml"
            )))
            .unwrap(),
            MembraneNormal::FromFile,
        );
    }
}
