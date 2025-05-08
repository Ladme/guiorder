use std::{fmt::Display, num::NonZeroUsize};

use eframe::egui::{self, RichText, Separator, Ui};
use gorder::{
    input::{Analysis, Axis, Frequency},
    prelude::AnalysisBuilder,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum AnalysisType {
    #[default]
    AAOrder,
    UAOrder,
    CGOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum LeafletClassification {
    #[default]
    None,
    Global,
    Local,
    Individual,
    Clustering,
    FromFile,
    FromNdx,
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
            LeafletClassification::FromNdx => write!(f, "NDX file(s)"),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct AAParams {
    heavy_atoms: String,
    hydrogens: String,
}

#[derive(Debug, Clone, Default)]
struct CGParams {
    beads: String,
}

#[derive(Debug, Clone, Default)]
struct UAParams {
    saturated: String,
    unsaturated: String,
    ignore: String,
}

#[derive(Debug, Clone, Default)]
struct AnalysisTypeParams {
    aa_params: AAParams,
    ua_params: UAParams,
    cg_params: CGParams,
}

#[derive(Debug, Clone, Default)]
struct OutputFiles {
    output_yaml: String,
    output_csv: String,
    output_tab: String,
    output_xvg: String,
}

#[derive(Debug, Clone, Default)]
struct LeafletGlobalParams {
    membrane: String,
    heads: String,
}

impl LeafletGlobalParams {
    fn sanity_check(&self) -> bool {
        !self.membrane.is_empty() && !self.heads.is_empty()
    }
}

#[derive(Debug, Clone)]
struct LeafletLocalParams {
    membrane: String,
    heads: String,
    radius: f32,
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

impl LeafletLocalParams {
    fn sanity_check(&self) -> bool {
        !self.membrane.is_empty() && !self.heads.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
struct LeafletIndividualParams {
    heads: String,
    methyls: String,
    radius: f32,
}

impl LeafletIndividualParams {
    fn sanity_check(&self) -> bool {
        !self.heads.is_empty() && !self.methyls.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
struct LeafletClusteringParams {
    heads: String,
}

impl LeafletClusteringParams {
    fn sanity_check(&self) -> bool {
        !self.heads.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
struct LeafletFromFileParams {
    file: String,
}

impl LeafletFromFileParams {
    fn sanity_check(&self) -> bool {
        !self.file.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
struct LeafletFromNdxParams {
    heads: String,
    ndx: String,
    upper_leaflet: String,
    lower_leaflet: String,
}

impl LeafletFromNdxParams {
    fn sanity_check(&self) -> bool {
        !self.heads.is_empty()
            && !self.ndx.is_empty()
            && !self.upper_leaflet.is_empty()
            && !self.lower_leaflet.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
struct LeafletClassificationParams {
    global_params: LeafletGlobalParams,
    local_params: LeafletLocalParams,
    individual_params: LeafletIndividualParams,
    clustering_params: LeafletClusteringParams,
    from_file_params: LeafletFromFileParams,
    from_ndx_params: LeafletFromNdxParams,
    frequency: Frequency,
    membrane_normal: Option<Axis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum RawFrequency {
    Once,
    #[default]
    Every,
    EveryN,
}

#[derive(Debug, Clone, Default)]
struct GuiAnalysis {
    structure: String,
    trajectory: String,
    ndx: String,
    bonds: String,
    analysis_type: AnalysisType,
    analysis_type_params: AnalysisTypeParams,
    output: OutputFiles,
    leaflet_classification_method: LeafletClassification,
    leaflet_classification_params: LeafletClassificationParams,
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
                    GuiAnalysis::specify_input_file(
                        &mut self.trajectory,
                        ui,
                        "Trajectory:  ",
                        "Path to a file containing the trajectory to analyze.",
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
                    self.specify_leaflet_classification(ui);

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
                        ui.add_space(54.0);

                        if Self::smart_button(ui, self.check_sanity(), "🔥 Run the analysis", "Perform the analysis using the specified options.", "Cannot run the analysis because some options are missing.").clicked() {
                            // todo; convert and run
                        };
                    });

                    ui.separator();
                });
        });
    }
}

impl GuiAnalysis {
    fn specify_input_file(
        target: &mut String,
        ui: &mut Ui,
        label: &str,
        hint: &str,
        required: bool,
    ) {
        ui.horizontal(|ui| {
            let label = ui
                .label(RichText::new(label).font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label(hint);
                })
                .on_hover_cursor(egui::CursorIcon::Help);
            if required && target.is_empty() {
                ui.add(
                    egui::TextEdit::singleline(target)
                        .background_color(egui::Color32::from_rgba_premultiplied(50, 0, 0, 50)),
                )
                .labelled_by(label.id);
            } else {
                ui.add(egui::TextEdit::singleline(target))
                    .labelled_by(label.id);
            }

            //ui.text_edit_singleline(target).labelled_by(label.id);
            if ui.button("📁").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    *target = path.display().to_string();
                }
            }
        });
    }

    fn specify_output_file(
        target: &mut String,
        ui: &mut Ui,
        label: &str,
        hint: &str,
        required: bool,
    ) {
        ui.horizontal(|ui| {
            let label = ui
                .label(RichText::new(label).font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label(hint);
                })
                .on_hover_cursor(egui::CursorIcon::Help);
            if required && target.is_empty() {
                ui.add(
                    egui::TextEdit::singleline(target)
                        .background_color(egui::Color32::from_rgba_premultiplied(50, 0, 0, 50)),
                )
                .labelled_by(label.id);
            } else {
                ui.add(egui::TextEdit::singleline(target))
                    .labelled_by(label.id);
            }

            if ui.button("📁").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    *target = path.display().to_string();
                }
            }
        });
    }

    fn specify_string(target: &mut String, ui: &mut Ui, label: &str, hint: &str, required: bool) {
        ui.horizontal(|ui| {
            let label = ui
                .label(RichText::new(label).font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label(hint);
                })
                .on_hover_cursor(egui::CursorIcon::Help);
            if required && target.is_empty() {
                ui.add(
                    egui::TextEdit::singleline(target)
                        .background_color(egui::Color32::from_rgba_premultiplied(50, 0, 0, 50)),
                )
                .labelled_by(label.id);
            } else {
                ui.add(egui::TextEdit::singleline(target))
                    .labelled_by(label.id);
            }
        });
    }

    fn specify_analysis_type(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Analysis type: ").font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label("Type of analysis to be performed.");
                })
                .on_hover_cursor(egui::CursorIcon::Help);
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

    fn specify_advanced_input(&mut self, ui: &mut Ui) {
        ui.collapsing("Advanced input options", |ui| {
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
        });
    }

    fn specify_advanced_output(&mut self, ui: &mut Ui) {
        ui.collapsing("Advanced output options", |ui| {
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
        });
    }

    fn toggle_radio<T: PartialEq + Clone>(
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

    fn smart_button(
        ui: &mut Ui,
        enabled: bool,
        text: &str,
        enabled_hint: &str,
        disabled_hint: &str,
    ) -> egui::Response {
        ui.add_enabled(enabled, egui::Button::new(text))
            .on_hover_ui(|ui| {
                ui.label(enabled_hint);
            })
            .on_disabled_hover_ui(|ui| {
                ui.label(disabled_hint);
            })
    }

    fn specify_frequency(frequency: &mut Frequency, ui: &mut Ui, label: &str) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label("Frequency of the leaflet assignment.");
                })
                .on_hover_cursor(egui::CursorIcon::Help);

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

    fn specify_leaflet_classification(&mut self, ui: &mut Ui) {
        let text = if self.check_leaflets_sanity() {
            RichText::new("Leaflet assignment options")
        } else {
            RichText::new("Leaflet assignment options")
                .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100))
        };

        ui.collapsing(text, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Assignment method: ").font(egui::FontId::monospace(12.0)))
                    .on_hover_ui(|ui| {
                        ui.label("Method to be used for assigning lipids into membrane leaflets.");
                    })
                    .on_hover_cursor(egui::CursorIcon::Help);
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
                    Self::specify_string(
                        &mut self.leaflet_classification_params.global_params.membrane,
                        ui,
                        "Membrane:    ",
                        "Selection of all lipid atoms forming the membrane.",
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.global_params.heads,
                        ui,
                        "Lipid heads: ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );
                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:   ",
                    );
                }
                LeafletClassification::Local => {
                    Self::specify_string(
                        &mut self.leaflet_classification_params.local_params.membrane,
                        ui,
                        "Membrane:    ",
                        "Selection of all lipid atoms forming the membrane.",
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.local_params.heads,
                        ui,
                        "Lipid heads: ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );

                    ui.horizontal(|ui| {
                        let label = ui
                            .label(
                                RichText::new("Radius:      ").font(egui::FontId::monospace(12.0)),
                            )
                            .on_hover_ui(|ui| {
                                ui.label("Radius of the cylinder [nm] for the calculation of local membrane center.");
                            })
                            .on_hover_cursor(egui::CursorIcon::Help);

                        ui.add(
                            egui::DragValue::new(
                                &mut self.leaflet_classification_params.local_params.radius,
                            )
                            .speed(0.1)
                            .range(0.0..=f32::MAX),
                        )
                        .labelled_by(label.id);
                    });

                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:   ",
                    );
                }
                LeafletClassification::Individual => {
                    Self::specify_string(
                        &mut self.leaflet_classification_params.individual_params.heads,
                        ui,
                        "Lipid heads:   ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.individual_params.methyls,
                        ui,
                        "Lipid methyls: ",
                        "Selection of lipid atoms representing ends of lipid tails.",
                        true,
                    );

                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:     ",
                    );
                }
                LeafletClassification::Clustering => {
                    Self::specify_string(
                        &mut self.leaflet_classification_params.clustering_params.heads,
                        ui,
                        "Lipid heads: ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );

                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:   ",
                    );
                }
                LeafletClassification::FromFile => {
                    Self::specify_input_file(
                        &mut self.leaflet_classification_params.from_file_params.file,
                        ui,
                        "Input file:  ",
                        "Path to a leaflet assignment file.",
                        true,
                    );

                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:   ",
                    );
                }
                LeafletClassification::FromNdx => {
                    Self::specify_string(
                        &mut self.leaflet_classification_params.from_ndx_params.heads,
                        ui,
                        "Lipid heads:   ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );
                    Self::specify_input_file(
                        &mut self.leaflet_classification_params.from_ndx_params.ndx,
                        ui,
                        "NDX file:      ",
                        "Path to an NDX file specifying the leaflets.",
                        true,
                    );
                    Self::specify_string(
                        &mut self
                            .leaflet_classification_params
                            .from_ndx_params
                            .upper_leaflet,
                        ui,
                        "Upper leaflet: ",
                        "Name of the NDX group containing atoms of the upper membrane leaflet.",
                        true,
                    );
                    Self::specify_string(
                        &mut self
                            .leaflet_classification_params
                            .from_ndx_params
                            .lower_leaflet,
                        ui,
                        "Lower leaflet: ",
                        "Name of the NDX group containing atoms of the lower membrane leaflet.",
                        true,
                    );

                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:     ",
                    );
                }
            });
        });
    }

    /// Check that all options required for the analysis have been provided.
    fn check_sanity(&self) -> bool {
        self.check_leaflets_sanity()
            && self.check_analysis_params_sanity()
            && !self.structure.is_empty()
            && !self.trajectory.is_empty()
            && !self.output.output_yaml.is_empty()
    }

    /// Check that all required options for leaflet assignment have been provided.
    fn check_leaflets_sanity(&self) -> bool {
        match self.leaflet_classification_method {
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
        }
    }

    /// Check that all required options for analysis type have been provided.
    fn check_analysis_params_sanity(&self) -> bool {
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
