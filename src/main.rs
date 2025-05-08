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
                            let mut input_yaml = if ui.button("📁 Import from YAML").clicked() {
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

                    GuiAnalysis::specify_input_file(&mut self.structure, ui, "Structure:   ", true);
                    GuiAnalysis::specify_input_file(
                        &mut self.trajectory,
                        ui,
                        "Trajectory:  ",
                        true,
                    );
                    GuiAnalysis::specify_output_file(
                        &mut self.output.output_yaml,
                        ui,
                        "Output YAML: ",
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
                        if ui.button("📁 Export to YAML").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                // todo; convert and export parameters
                            }
                        }

                        ui.add_space(54.0);
                        ui.separator();
                        ui.add_space(54.0);

                        if ui.button("🔥 Run the analysis").clicked() {
                            // todo; convert and run
                        }
                    });

                    ui.separator();
                });
        });
    }
}

impl GuiAnalysis {
    fn specify_input_file(target: &mut String, ui: &mut Ui, label: &str, required: bool) {
        ui.horizontal(|ui| {
            let label = ui.label(RichText::new(label).font(egui::FontId::monospace(12.0)));
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

    fn specify_output_file(target: &mut String, ui: &mut Ui, label: &str, required: bool) {
        ui.horizontal(|ui| {
            let label = ui.label(RichText::new(label).font(egui::FontId::monospace(12.0)));
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

    fn specify_string(target: &mut String, ui: &mut Ui, label: &str, required: bool) {
        ui.horizontal(|ui| {
            let label = ui.label(RichText::new(label).font(egui::FontId::monospace(12.0)));
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
            ui.label(RichText::new("Analysis type: ").font(egui::FontId::monospace(12.0)));
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
                    true,
                );
                Self::specify_string(
                    &mut self.analysis_type_params.aa_params.hydrogens,
                    ui,
                    "Hydrogens:   ",
                    true,
                );
            }

            AnalysisType::UAOrder => {
                Self::specify_string(
                    &mut self.analysis_type_params.ua_params.saturated,
                    ui,
                    "Saturated carbons:   ",
                    true,
                );
                Self::specify_string(
                    &mut self.analysis_type_params.ua_params.unsaturated,
                    ui,
                    "Unsaturated carbons: ",
                    true,
                );
                Self::specify_string(
                    &mut self.analysis_type_params.ua_params.ignore,
                    ui,
                    "Ignore:              ",
                    false,
                );
            }
            AnalysisType::CGOrder => {
                Self::specify_string(
                    &mut self.analysis_type_params.cg_params.beads,
                    ui,
                    "Beads: ",
                    true,
                );
            }
        });
    }

    fn specify_advanced_input(&mut self, ui: &mut Ui) {
        ui.collapsing("Advanced input options", |ui| {
            Self::specify_input_file(&mut self.bonds, ui, "Bonds file:   ", false);
            Self::specify_input_file(&mut self.ndx, ui, "NDX file:     ", false);
        });
    }

    fn specify_advanced_output(&mut self, ui: &mut Ui) {
        ui.collapsing("Advanced output options", |ui| {
            Self::specify_output_file(&mut self.output.output_csv, ui, "Output CSV:   ", false);
            Self::specify_output_file(&mut self.output.output_tab, ui, "Output Table: ", false);
            Self::specify_output_file(&mut self.output.output_xvg, ui, "Output XVG:   ", false);
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

    fn specify_frequency(frequency: &mut Frequency, ui: &mut Ui, label: &str) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).font(egui::FontId::monospace(12.0)));

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
                ui.label(RichText::new("Assignment method: ").font(egui::FontId::monospace(12.0)));
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
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.global_params.heads,
                        ui,
                        "Lipid heads: ",
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
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.local_params.heads,
                        ui,
                        "Lipid heads: ",
                        true,
                    );

                    ui.horizontal(|ui| {
                        let label = ui.label(
                            RichText::new("Radius:      ").font(egui::FontId::monospace(12.0)),
                        );

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
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.individual_params.methyls,
                        ui,
                        "Lipid methyls: ",
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
                        true,
                    );
                    Self::specify_input_file(
                        &mut self.leaflet_classification_params.from_ndx_params.ndx,
                        ui,
                        "NDX file:      ",
                        true,
                    );
                    Self::specify_string(
                        &mut self
                            .leaflet_classification_params
                            .from_ndx_params
                            .upper_leaflet,
                        ui,
                        "Upper leaflet: ",
                        true,
                    );
                    Self::specify_string(
                        &mut self
                            .leaflet_classification_params
                            .from_ndx_params
                            .lower_leaflet,
                        ui,
                        "Lower leaflet: ",
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
}
