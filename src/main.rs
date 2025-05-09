use std::{fmt::Display, num::NonZeroUsize};

use eframe::egui::{self, Color32, CursorIcon, Response, RichText, Sense, Separator, Ui};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AnalysisType {
    #[default]
    AAOrder,
    UAOrder,
    CGOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    ndx: Vec<String>,
    upper_leaflet: String,
    lower_leaflet: String,
}

impl LeafletFromNdxParams {
    fn sanity_check(&self) -> bool {
        !self.heads.is_empty()
            && !self.ndx.iter().any(|file| file.is_empty())
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
    membrane_normal: Option<MembraneNormal>,
}

#[derive(Debug, Clone)]
struct DynamicNormalParams {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RawFrequency {
    Once,
    #[default]
    Every,
    EveryN,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum MembraneNormal {
    X,
    Y,
    #[default]
    Z,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum OrderMapDimension {
    #[default]
    Auto,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Plane {
    Unknown,
    #[default]
    XY,
    YZ,
    XZ,
}

#[derive(Debug, Clone)]
struct ManualDimensions {
    start: f32,
    end: f32,
}

impl Default for ManualDimensions {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 10.0,
        }
    }
}

#[derive(Debug, Clone)]
struct OrderMapsParams {
    calculate_maps: bool,
    output_directory: String,
    plane: Option<Plane>,
    bin_size: [f32; 2],
    dimensions: [OrderMapDimension; 2],
    x_manual: ManualDimensions,
    y_manual: ManualDimensions,
    min_samples: usize,
}

impl Default for OrderMapsParams {
    fn default() -> Self {
        Self {
            calculate_maps: false,
            output_directory: String::new(),
            plane: None,
            bin_size: [0.1, 0.1],
            dimensions: [OrderMapDimension::default(), OrderMapDimension::default()],
            x_manual: ManualDimensions::default(),
            y_manual: ManualDimensions::default(),
            min_samples: 1,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct GuiAnalysis {
    structure: String,
    trajectory: Vec<String>,
    ndx: String,
    bonds: String,
    analysis_type: AnalysisType,
    analysis_type_params: AnalysisTypeParams,
    output: OutputFiles,
    leaflet_classification_method: LeafletClassification,
    leaflet_classification_params: LeafletClassificationParams,
    membrane_normal: MembraneNormal,
    dynamic_normal_params: DynamicNormalParams,
    ordermaps_params: OrderMapsParams,
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
                    GuiAnalysis::specify_multiple_input_files(
                        &mut self.trajectory,
                        ui,
                        "Trajectory:  ",
                        "Path to a file containing the trajectory to analyze. Provide multiple files by clicking the '+' button or by selecting them interactively.",
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
                    self.specify_membrane_normal(ui);
                    self.specify_leaflet_classification(ui);
                    self.specify_ordermaps(ui);

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
                        ui.add_space(46.0);

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
            if ui
                .button("📁")
                .on_hover_ui(|ui| {
                    ui.label("Select the file interactively.");
                })
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    *target = path.display().to_string();
                }
            }
        });
    }

    fn specify_multiple_input_files(
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
                let label = ui
                    .label(RichText::new(label).font(egui::FontId::monospace(12.0)))
                    .on_hover_ui(|ui| {
                        ui.label(hint);
                    })
                    .on_hover_cursor(egui::CursorIcon::Help);

                if target.is_empty() {
                    target.push(String::new());
                }

                if required && target[0].is_empty() {
                    ui.add(
                        egui::TextEdit::singleline(&mut target[0])
                            .background_color(egui::Color32::from_rgba_premultiplied(50, 0, 0, 50)),
                    )
                    .labelled_by(label.id);
                } else {
                    ui.add(egui::TextEdit::singleline(&mut target[0]))
                        .labelled_by(label.id);
                }

                if ui
                    .add_enabled(
                        !target.iter().any(|x| x.is_empty()),
                        egui::Button::new("➕"),
                    )
                    .on_hover_ui(|ui| {
                        ui.label("Select another file.");
                    })
                    .on_disabled_hover_ui(|ui| {
                        ui.label(
                            "Cannot select another file because a previous filename is missing.",
                        );
                    })
                    .clicked()
                {
                    target.push(String::new());
                }

                if ui
                    .button("📁")
                    .on_hover_ui(|ui| {
                        ui.label("Select the file(s) interactively.");
                    })
                    .clicked()
                {
                    if let Some(paths) = rfd::FileDialog::new().pick_files() {
                        target.clear();
                        for file in paths {
                            target.push(file.display().to_string())
                        }
                    }
                }
            });
        } else {
            let text = if !target.iter().any(|file| file.is_empty()) {
                RichText::new(label).font(egui::FontId::monospace(12.0))
            } else {
                RichText::new(label)
                    .font(egui::FontId::monospace(12.0))
                    .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100))
            };

            ui.horizontal(|ui| {
                egui::CollapsingHeader::new(text)
                .default_open(true)
                .show(ui, |ui| {
                    let mut index_to_remove = None;
                    let mut move_up = None;
                    let mut move_down = None;
                    let last_index = target.len() - 1;
                    let no_empty = !target.iter().any(|x| x.is_empty());
                    for (i, item) in target.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if required && item.is_empty() {
                                ui.add(
                                    egui::TextEdit::singleline(item)
                                        .background_color(egui::Color32::from_rgba_premultiplied(50, 0, 0, 50)),
                                );
                            } else {
                                ui.add(egui::TextEdit::singleline(item));
                            }
                            
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
                        if ui
                        .add_enabled(no_empty, egui::Button::new("➕"))
                        .on_hover_ui(|ui| {
                            ui.label("Select another file.");
                        })
                        .on_disabled_hover_ui(|ui| {
                            ui.label(
                "Cannot select another file because a previous filename is missing.",
                        );
                        })
                        .clicked()
                    {
                        target.push(String::new());
                    }
                    });
                    
                });

                if ui
                    .button("📁")
                    .on_hover_ui(|ui| {
                        ui.label("Select the files interactively. This will replace all already selected files!");
                    })
                    .clicked()
                {
                    if let Some(paths) = rfd::FileDialog::new().pick_files() {
                        target.clear();
                        for file in paths {
                            target.push(file.display().to_string())
                        }
                    }
                }
            });
        }
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

            if ui
                .button("📁")
                .on_hover_ui(|ui| {
                    ui.label("Specify the file interactively.");
                })
                .clicked()
            {
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
                    ui.label("Type of order parameters to calculate.");
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
        ui.collapsing(
            RichText::new("Advanced input").font(egui::FontId::monospace(12.0)),
            |ui| {
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
            },
        );
    }

    fn specify_advanced_output(&mut self, ui: &mut Ui) {
        ui.collapsing(
            RichText::new("Advanced output").font(egui::FontId::monospace(12.0)),
            |ui| {
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
            },
        );
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

    fn specify_leaflet_membrane_normal(&mut self, ui: &mut Ui, label: &str) {
        let raw_normal = if let Some(x) = self.leaflet_classification_params.membrane_normal {
            x
        } else {
            self.membrane_normal
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(label).font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label("Membrane normal used for the leaflet classification. Can be decoupled from the global membrane normal.");
                })
                .on_hover_cursor(egui::CursorIcon::Help);

            if ui.add(egui::RadioButton::new(raw_normal == MembraneNormal::X, "x")).clicked() {
                self.leaflet_classification_params.membrane_normal = Some(MembraneNormal::X);
            }

            if ui.add(egui::RadioButton::new(raw_normal == MembraneNormal::Y, "y")).clicked() {
                self.leaflet_classification_params.membrane_normal = Some(MembraneNormal::Y);
            }

            if ui.add(egui::RadioButton::new(raw_normal == MembraneNormal::Z, "z")).clicked() {
                self.leaflet_classification_params.membrane_normal = Some(MembraneNormal::Z);
            }

            if raw_normal == MembraneNormal::Dynamic {
                ui.label(RichText::new("❗").color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)));
            }
        });
    }

    fn specify_leaflet_classification(&mut self, ui: &mut Ui) {
        let text = if self.check_leaflets_sanity() {
            RichText::new("Leaflet assignment").font(egui::FontId::monospace(12.0))
        } else {
            RichText::new("Leaflet assignment")
                .font(egui::FontId::monospace(12.0))
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
                        "Membrane:        ",
                        "Selection of all lipid atoms forming the membrane.",
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.global_params.heads,
                        ui,
                        "Lipid heads:     ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );
                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:       ",
                    );
                    self.specify_leaflet_membrane_normal(ui, "Membrane normal: ");
                }
                LeafletClassification::Local => {
                    Self::specify_string(
                        &mut self.leaflet_classification_params.local_params.membrane,
                        ui,
                        "Membrane:        ",
                        "Selection of all lipid atoms forming the membrane.",
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.local_params.heads,
                        ui,
                        "Lipid heads:     ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );

                    ui.horizontal(|ui| {
                        let label = ui
                            .label(
                                RichText::new("Radius:          ").font(egui::FontId::monospace(12.0)),
                            )
                            .on_hover_ui(|ui| {
                                ui.label("Radius of the cylinder for the calculation of local membrane center.");
                            })
                            .on_hover_cursor(egui::CursorIcon::Help);

                        ui.add(
                            egui::DragValue::new(
                                &mut self.leaflet_classification_params.local_params.radius,
                            )
                            .speed(0.1)
                            .range(0.0..=f32::MAX)
                            .suffix(" nm"),
                        )
                        .labelled_by(label.id);
                    });

                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:       ",
                    );

                    self.specify_leaflet_membrane_normal(ui, "Membrane normal: ");
                }
                LeafletClassification::Individual => {
                    Self::specify_string(
                        &mut self.leaflet_classification_params.individual_params.heads,
                        ui,
                        "Lipid heads:     ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.individual_params.methyls,
                        ui,
                        "Lipid methyls:   ",
                        "Selection of lipid atoms representing ends of lipid tails.",
                        true,
                    );

                    Self::specify_frequency(
                        &mut self.leaflet_classification_params.frequency,
                        ui,
                        "Frequency:       ",
                    );

                    self.specify_leaflet_membrane_normal(ui, "Membrane normal: ");
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
                    Self::specify_multiple_input_files(
                        &mut self.leaflet_classification_params.from_ndx_params.ndx,
                        ui,
                        "NDX files: ",
                        "Path to NDX files specifying the leaflets.",
                        true,
                    );
                    Self::specify_string(
                        &mut self.leaflet_classification_params.from_ndx_params.heads,
                        ui,
                        "Lipid heads:   ",
                        "Selection of lipid atoms representing lipid heads. One atom per molecule!",
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

    fn specify_membrane_normal(&mut self, ui: &mut Ui) {
        let text = if self.check_membrane_normal_sanity() {
            RichText::new("Membrane normal").font(egui::FontId::monospace(12.0))
        } else {
            RichText::new("Membrane normal")
                .font(egui::FontId::monospace(12.0))
                .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100))
        };

        ui.collapsing(text, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Membrane normal: ").font(egui::FontId::monospace(12.0)))
                    .on_hover_ui(|ui| {
                        ui.label("Direction of the membrane normal.");
                    })
                    .on_hover_cursor(egui::CursorIcon::Help);

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
                        let label = ui
                            .label(
                                RichText::new("Radius:      ").font(egui::FontId::monospace(12.0)),
                            )
                            .on_hover_ui(|ui| {
                                ui.label("Radius of the scanning sphere for identification of nearby lipid heads.");
                            })
                            .on_hover_cursor(egui::CursorIcon::Help);

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
        });
    }

    fn specify_ordermaps(&mut self, ui: &mut Ui) {
        let text = if self.check_ordermaps_sanity() {
            RichText::new("Order parameter maps").font(egui::FontId::monospace(12.0))
        } else {
            RichText::new("Order parameter maps")
                .font(egui::FontId::monospace(12.0))
                .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100))
        };

        ui.collapsing(text, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Construct ordermaps: ").font(egui::FontId::monospace(12.0)))
                    .on_hover_ui(|ui| {
                        ui.label("Should the ordermaps be constructed?");
                    })
                    .on_hover_cursor(egui::CursorIcon::Help);

                ui.checkbox(&mut self.ordermaps_params.calculate_maps, "");
            });

            if !self.ordermaps_params.calculate_maps {
                return;
            }

            Self::specify_string(
                &mut self.ordermaps_params.output_directory, 
                ui, 
                "Directory:  ", 
                "Name of a directory for saving ordermaps. Directory does not have to already exist.", 
                true
            );

            let raw_plane = if let Some(plane) = self.ordermaps_params.plane {
                plane
            } else {
                match self.membrane_normal {
                    MembraneNormal::X => Plane::YZ,
                    MembraneNormal::Y => Plane::XZ,
                    MembraneNormal::Z => Plane::XY,
                    MembraneNormal::Dynamic => Plane::Unknown,
                }
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new("Plane:      ").font(egui::FontId::monospace(12.0)))
                    .on_hover_ui(|ui| {
                        ui.label("Plane in which the ordermaps will be constructed.");
                    })
                    .on_hover_cursor(egui::CursorIcon::Help);

                if ui.add(egui::RadioButton::new(raw_plane == Plane::XY, "xy")).clicked() {
                    self.ordermaps_params.plane = Some(Plane::XY);
                }
    
                if ui.add(egui::RadioButton::new(raw_plane == Plane::XZ, "xz")).clicked() {
                    self.ordermaps_params.plane = Some(Plane::XZ);
                }

                if ui.add(egui::RadioButton::new(raw_plane == Plane::YZ, "yz")).clicked() {
                    self.ordermaps_params.plane = Some(Plane::YZ);
                }

                if raw_plane == Plane::Unknown {
                    ui.label(RichText::new("❗").color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)));
                }
            });

            let (dim_1, dim_2) = match raw_plane {
                Plane::XY => ("X-dimension", "Y-dimension"),
                Plane::YZ => ("Z-dimension", "Y-dimension"),
                Plane::XZ => ("X-dimension", "Z-dimension"),
                Plane::Unknown => ("Unknown dimension", "Unknown dimension"),
            };

            ui.vertical(|ui| {
                ui.label(RichText::new("Maps size:  ").font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label("Size of the ordermaps.");
                })
                .on_hover_cursor(egui::CursorIcon::Help);

                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!(" {dim_1}: ")).font(egui::FontId::monospace(12.0)))
                    .on_hover_ui(|ui| {ui.label(format!("Size of the ordermaps along the {}.", dim_1.to_lowercase()));})
                    .on_hover_cursor(egui::CursorIcon::Help);

                    ui.radio_value(&mut self.ordermaps_params.dimensions[0], OrderMapDimension::Auto, "automatic");
                    ui.radio_value(&mut self.ordermaps_params.dimensions[0], OrderMapDimension::Manual, "manual");

                    if self.ordermaps_params.dimensions[0] == OrderMapDimension::Manual {
                        ui.add(
                            egui::DragValue::new(
                                &mut self.ordermaps_params.x_manual.start,
                            )
                            .speed(0.1)
                            .range(-f32::MAX..=self.ordermaps_params.x_manual.end)
                            .suffix(" nm"),
                        ).on_hover_ui(|ui| {
                            ui.label("start");
                        });

                        ui.add(
                            egui::DragValue::new(
                                &mut self.ordermaps_params.x_manual.end,
                            )
                            .speed(0.1)
                            .range(self.ordermaps_params.x_manual.start..=f32::MAX)
                            .suffix(" nm"),
                        ).on_hover_ui(|ui| {
                            ui.label("end");
                        });
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!(" {dim_2}: ")).font(egui::FontId::monospace(12.0)))
                    .on_hover_ui(|ui| {ui.label(format!("Size of the ordermaps along the {}.", dim_2.to_lowercase()));})
                    .on_hover_cursor(egui::CursorIcon::Help);

                    ui.radio_value(&mut self.ordermaps_params.dimensions[1], OrderMapDimension::Auto, "automatic");
                    ui.radio_value(&mut self.ordermaps_params.dimensions[1], OrderMapDimension::Manual, "manual");

                    if self.ordermaps_params.dimensions[1] == OrderMapDimension::Manual {
                        ui.add(
                            egui::DragValue::new(
                                &mut self.ordermaps_params.y_manual.start,
                            )
                            .speed(0.1)
                            .range(-f32::MAX..=self.ordermaps_params.y_manual.end)
                            .suffix(" nm"),
                        ).on_hover_ui(|ui| {
                            ui.label("start");
                        });

                        ui.add(
                            egui::DragValue::new(
                                &mut self.ordermaps_params.y_manual.end,
                            )
                            .speed(0.1)
                            .range(self.ordermaps_params.y_manual.start..=f32::MAX)
                            .suffix(" nm"),
                        ).on_hover_ui(|ui| {
                            ui.label("end");
                        });
                    }
                });
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Bin size:   ").font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label("Size of the bins of the ordermap.");
                })
                .on_hover_cursor(egui::CursorIcon::Help);

                ui.add(
                    egui::DragValue::new(
                        &mut self.ordermaps_params.bin_size[0],
                    )
                    .speed(0.01)
                    .range(0.0..=f32::MAX)
                    .suffix(" nm"),
                ).on_hover_ui(|ui| {
                    ui.label(dim_1.to_lowercase());
                });

                ui.add(
                    egui::DragValue::new(
                        &mut self.ordermaps_params.bin_size[1],
                    )
                    .speed(0.01)
                    .range(0.0..=f32::MAX)
                    .suffix(" nm"),
                ).on_hover_ui(|ui| {
                    ui.label(dim_2.to_lowercase());
                });
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Min samples:").font(egui::FontId::monospace(12.0)))
                .on_hover_ui(|ui| {
                    ui.label("Minimum number of samples required in a bin to calculate the order parameter.");
                })
                .on_hover_cursor(egui::CursorIcon::Help);

                ui.add(
                    egui::DragValue::new(
                        &mut self.ordermaps_params.min_samples,
                    )
                    .speed(2.5)
                    .range(1..=usize::MAX),
                );
            });

            
        });
    }

    /// Check that all options required for the analysis have been provided.
    fn check_sanity(&self) -> bool {
        self.check_leaflets_sanity()
            && self.check_analysis_params_sanity()
            && !self.structure.is_empty()
            && !self.trajectory.iter().any(|file| file.is_empty())
            && !self.output.output_yaml.is_empty()
            && self.check_membrane_normal_sanity()
            && self.check_ordermaps_sanity()
    }

    /// Check that all required options for leaflet assignment have been provided.
    fn check_leaflets_sanity(&self) -> bool {
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

    /// Check that all required options for membrane normal specification have been provided.
    fn check_membrane_normal_sanity(&self) -> bool {
        match self.membrane_normal {
            MembraneNormal::Dynamic => !self.dynamic_normal_params.heads.is_empty(),
            _ => true,
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

    fn check_ordermaps_sanity(&self) -> bool {
        !self.ordermaps_params.calculate_maps || (
            !self.ordermaps_params.output_directory.is_empty() && (self.ordermaps_params.plane.is_some() || self.membrane_normal != MembraneNormal::Dynamic))
    }
}
