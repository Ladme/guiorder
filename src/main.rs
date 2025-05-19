// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use colored::Colorize;
use common::GuiAnalysis;
use eframe::egui::{self, RichText, Ui};
use gorder::colog_info;
use leaflets::{LeafletClassification, LeafletClassificationParams};
use window::Windows;

mod analysis_types;
mod common;
mod convert;
mod error;
mod estimate_error;
mod frame_selection;
mod geometry;
mod leaflets;
mod membrane_normal;
mod ordermaps;
mod other_options;
mod window;

pub const GUIORDER_VERSION: &str = env!("CARGO_PKG_VERSION");
const LINE_SPACING: f32 = 10.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([465.0, 640.0])
            .with_resizable(false),
        ..Default::default()
    };

    colog::init();

    eframe::run_native(
        &format!("guiorder v{}", GUIORDER_VERSION),
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::from(GuiOrderApp::default()))
        }),
    )
}

/// Structure handling the entire application.
#[derive(Debug, Default)]
#[allow(clippy::type_complexity)]
pub(crate) struct GuiOrderApp {
    analysis: GuiAnalysis,
    windows: Windows,
    /// Analysis running?
    running: Arc<Mutex<bool>>,
    thread_handle: Mutex<Option<JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>>,
}

impl eframe::App for GuiOrderApp {
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
                    self.import_yaml_button(ui);
                    ui.separator();

                    GuiAnalysis::specify_input_file(
                        &mut self.analysis.structure,
                        ui,
                        "Structure:   ",
                        "Path to a file containing the structure of the system.",
                        true,
                    );
                    GuiAnalysis::specify_multiple_input_files(
                        &mut self.analysis.trajectory,
                        ui,
                        "Trajectory:  ",
                        "Path to a file containing the trajectory to analyze. Provide multiple files by clicking the '+' button or by selecting them interactively.",
                        true,
                    );
                    GuiAnalysis::specify_output_file(
                        &mut self.analysis.output.output_yaml,
                        ui,
                        "Output YAML: ",
                        "Path to an output YAML file where the full results of the analysis will be saved.",
                        true,
                    );

                    ui.separator();
                    self.analysis.specify_analysis_type(ui);
                    ui.separator();

                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_advanced_input(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_advanced_output(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_frame_selection(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_membrane_normal(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_leaflet_classification(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_ordermaps(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_geometry(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_estimate_error(ui);
                    ui.add_space(LINE_SPACING);
                    self.analysis.specify_other_options(ui);
                    ui.add_space(LINE_SPACING);

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add_space(54.0);
                        if GuiAnalysis::smart_button(
                            ui,
                            self.analysis.check_sanity(),
                            false,
                            "📁 Export to YAML",
                            "Export analysis options into a YAML configuration file.",
                            "Cannot export analysis options because some are missing.",
                            "This should never appear.",
                        ).clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                self.export_to_yaml(path);
                            }
                        }

                        ui.add_space(54.0);
                        ui.separator();
                        ui.add_space(46.0);

                        let hint = if self.analysis.other_params.n_threads >= 2 {
                            format!("Perform the analysis using {} threads.", self.analysis.other_params.n_threads)
                        } else {
                            "Perform the analysis using 1 thread.".to_string()
                        };

                        if GuiAnalysis::smart_button(
                            ui,
                            self.analysis.check_sanity(),
                            *self.running.lock().unwrap(),
                            "🔥 Run the analysis",
                            &hint,
                            "Cannot run the analysis because some options are missing.", 
                            "Analysis is already running."
                        ).clicked() {
                            self.run_analysis();
                        };
                    });

                    ui.separator();

                    // display that the analysis is running
                    if *self.running.lock().unwrap() {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.spinner();
                            ui.label(
                                RichText::new("Analysis is running. See the terminal for more details.")
                                    .font(egui::FontId::monospace(12.0))
                            );
                        });
                    // check for errors during the analysis
                    } else {
                        let handle = self.thread_handle.lock().unwrap().take();
                        match handle {
                            None => (), // no result, do nothing
                            Some(handle) => {
                                match handle.join().unwrap() {
                                    Ok(_) => {
                                        Self::display_result(true, self.analysis.other_params.silent);
                                        self.open_success_window("Analysis finished successfully.");
                                    }
                                    Err(e) => {
                                        log::error!("{}", e);
                                        Self::display_result(false, self.analysis.other_params.silent);
                                        self.open_error_window(e);
                                    }
                                }
                            }
                        }
                    }

                    // render windows
                    self.windows.render(ctx);
                });
        });
    }
}

/// Paths to all specified output files.
#[derive(Debug, Clone, Default)]
struct OutputFiles {
    output_yaml: String,
    output_csv: String,
    output_tab: String,
    output_xvg: String,
}

impl From<&gorder::input::Analysis> for OutputFiles {
    fn from(value: &gorder::input::Analysis) -> Self {
        Self {
            output_yaml: value.output_yaml().clone().unwrap_or_default(),
            output_csv: value.output_csv().clone().unwrap_or_default(),
            output_tab: value.output_tab().clone().unwrap_or_default(),
            output_xvg: value.output_xvg().clone().unwrap_or_default(),
        }
    }
}

impl GuiOrderApp {
    /// Create a button for importing parameters from a YAML file.
    fn import_yaml_button(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical_centered(|ui| {
                let input_yaml = if ui
                    .button("📁 Import from YAML")
                    .on_hover_ui(|ui| {
                        ui.label("Load a YAML configuration file.");
                    })
                    .clicked()
                {
                    rfd::FileDialog::new()
                        .set_directory(".")
                        .pick_file()
                        .map(|path| path.display().to_string())
                } else {
                    None
                };

                if let Some(input) = input_yaml {
                    self.import_yaml(&input);
                }
            });
        });
    }

    /// Import parameters from a yaml file.
    fn import_yaml(&mut self, input: &str) {
        match gorder::input::Analysis::from_file(input) {
            Err(e) => self.open_error_window(Box::from(e)),
            Ok(analysis) => match analysis.try_into() {
                Err(e) => self.open_error_window(Box::from(e)),
                Ok(converted) => {
                    self.analysis = converted;
                }
            },
        }
    }

    /// Convert the GuiAnalysis to gorder analysis structure and run the analysis.
    fn run_analysis(&mut self) {
        let converted = match gorder::input::Analysis::try_from(&self.analysis) {
            Err(e) => {
                self.open_error_window(Box::from(e));
                return;
            }
            Ok(x) => x,
        };

        if !self.analysis.other_params.silent {
            log::set_max_level(log::LevelFilter::Info);
            let header = format!(">>> GORDER v{} <<<", gorder::GORDER_VERSION).bold();
            println!("\n{}\n", header);
        } else {
            log::set_max_level(log::LevelFilter::Error);
        }

        colog_info!(
            "Analysis parameters supplied by {}.",
            format!("guiorder v{}", GUIORDER_VERSION)
        );

        let is_running = Arc::clone(&self.running);
        *self.running.lock().unwrap() = true;

        let handle = std::thread::spawn(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                match converted.run() {
                    Err(e) => {
                        *is_running.lock().unwrap() = false;
                        Err(e)
                    }
                    Ok(results) => match results.write() {
                        Err(e) => {
                            *is_running.lock().unwrap() = false;
                            Err(Box::from(e))
                        }
                        Ok(_) => {
                            *is_running.lock().unwrap() = false;
                            Ok(())
                        }
                    },
                }
            },
        );

        *self.thread_handle.lock().unwrap() = Some(handle);
    }

    /// Convert the GuiAnalysis to gorder analysis structure and export it to an output yaml file.
    fn export_to_yaml(&mut self, output: PathBuf) {
        let converted = match gorder::input::Analysis::try_from(&self.analysis) {
            Err(e) => {
                self.open_error_window(Box::from(e));
                return;
            }
            Ok(x) => x,
        };

        let file = match File::create(&output) {
            Err(e) => {
                self.open_error_window(Box::from(e));
                return;
            }
            Ok(x) => x,
        };
        let mut writer = BufWriter::new(file);

        writeln!(
            writer,
            "# Analysis options generated by 'guiorder v{}'.",
            GUIORDER_VERSION
        )
        .unwrap();

        match serde_yaml::to_writer(&mut writer, &converted) {
            Err(e) => {
                self.open_error_window(Box::from(e));
            }
            Ok(_) => self.open_success_window(&format!(
                "Successfully exported analysis options into a configuration YAML file '{}'.",
                output.to_str().unwrap()
            )),
        }
    }

    /// Display the result of the analysis.
    fn display_result(result: bool, silent: bool) {
        if silent {
            return;
        }

        match result {
            true => {
                let prefix = format!(
                    "{}{}{}",
                    "[".to_string().blue().bold(),
                    "✔".to_string().bright_green().bold(),
                    "]".to_string().blue().bold()
                );
                let message = "ANALYSIS COMPLETED".to_string().bright_green().bold();
                println!("{} {}\n", prefix, message);
            }
            false => {
                let prefix = format!(
                    "{}{}{}",
                    "[".to_string().blue().bold(),
                    "✖".to_string().red().bold(),
                    "]".to_string().blue().bold()
                );
                let message = "ANALYSIS FAILED".to_string().red().bold();
                println!("{} {}\n", prefix, message);
            }
        }
    }
}

impl GuiAnalysis {
    /// Specify optional paths to a bonds file and an NDX file.
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

    /// Specify paths to CSV, Table, and XVG output.
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

    /// Check that all options required for the analysis have been provided.
    fn check_sanity(&self) -> bool {
        self.check_leaflets_sanity()
            && self.check_analysis_params_sanity()
            && !self.structure.is_empty()
            && !self.trajectory.iter().any(|file| file.is_empty())
            && !self.output.output_yaml.is_empty()
            && self.check_membrane_normal_sanity()
            && self.check_ordermaps_sanity()
            && self.check_geometry_sanity()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader};

    use approx::assert_relative_eq;
    use tempfile::NamedTempFile;

    use super::*;

    fn assert_eq_order(a: &str, b: &str, skip: usize) {
        let (file_a, file_b) = match (File::open(a), File::open(b)) {
            (Ok(f1), Ok(f2)) => (f1, f2),
            _ => panic!("One or both files do not exist."),
        };

        let mut lines_a = BufReader::new(file_a).lines().skip(skip);
        let mut lines_b = BufReader::new(file_b).lines().skip(skip);

        loop {
            match (lines_a.next(), lines_b.next()) {
                (Some(Ok(line_a)), Some(Ok(line_b))) => assert_lines(&line_a, &line_b),
                (None, None) => break,
                _ => panic!("Files have different number of lines"),
            }
        }
    }

    fn assert_lines(line_a: &str, line_b: &str) {
        let mut line_a_split = line_a.split_whitespace();
        let mut line_b_split = line_b.split_whitespace();

        loop {
            match (line_a_split.next(), line_b_split.next()) {
                (Some(item_a), Some(item_b)) => assert_items(item_a, item_b),
                (None, None) => break,
                _ => panic!("Lines do not match: {} vs. {}", line_a, line_b),
            }
        }
    }

    fn assert_items(item_a: &str, item_b: &str) {
        match (item_a.parse::<f32>(), item_b.parse::<f32>()) {
            (Ok(z1), Ok(z2)) if z1.is_nan() && z2.is_nan() => (),
            (Ok(z1), Ok(z2)) => assert_relative_eq!(z1, z2, epsilon = 2e-4),
            (Err(_), Err(_)) => assert_eq!(
                item_a, item_b,
                "Items do not match: {} vs {}",
                item_a, item_b
            ),
            _ => panic!("Invalid or mismatched items: {} vs {}", item_a, item_b),
        }
    }

    fn assert_eq_csv(a: &str, b: &str, skip: usize) {
        let (file_a, file_b) = match (File::open(a), File::open(b)) {
            (Ok(f1), Ok(f2)) => (f1, f2),
            _ => panic!("One or both files do not exist."),
        };

        let mut lines_a = BufReader::new(file_a).lines().skip(skip);
        let mut lines_b = BufReader::new(file_b).lines().skip(skip);

        loop {
            match (lines_a.next(), lines_b.next()) {
                (Some(Ok(line_a)), Some(Ok(line_b))) => assert_lines_csv(&line_a, &line_b),
                (None, None) => break,
                _ => panic!("Files have different number of lines"),
            }
        }
    }

    fn assert_lines_csv(line_a: &str, line_b: &str) {
        let mut line_a_split = line_a.split(",");
        let mut line_b_split = line_b.split(",");

        loop {
            match (line_a_split.next(), line_b_split.next()) {
                (Some(item_a), Some(item_b)) => assert_items(item_a, item_b),
                (None, None) => break,
                _ => panic!("Lines do not match: {} vs. {}", line_a, line_b),
            }
        }
    }

    fn assert_eq_maps(a: &str, b: &str, skip: usize) {
        let (file_a, file_b) = match (File::open(a), File::open(b)) {
            (Ok(f1), Ok(f2)) => (f1, f2),
            _ => panic!("One or both files do not exist."),
        };

        let mut lines_a = BufReader::new(file_a).lines().skip(skip);
        let mut lines_b = BufReader::new(file_b).lines().skip(skip);

        loop {
            match (lines_a.next(), lines_b.next()) {
                (Some(Ok(line_a)), Some(Ok(line_b))) => {
                    let is_data = line_a
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<f32>().ok())
                        .is_some();

                    if is_data {
                        let p: Vec<_> = line_a.split_whitespace().collect();
                        let q: Vec<_> = line_b.split_whitespace().collect();
                        assert_eq!(p.len(), 3, "Data line must have 3 columns");
                        assert_eq!(q.len(), 3, "Data line must have 3 columns");

                        assert_eq!(p[0], q[0], "First columns differ");
                        assert_eq!(p[1], q[1], "Second columns differ");

                        assert_items(p[2], q[2]);
                    } else {
                        assert_eq!(line_a, line_b, "Non-data lines differ");
                    }
                }
                (None, None) => break,
                _ => panic!("Files have different number of lines"),
            }
        }
    }

    fn diff_files_ignore_first(file1: &str, file2: &str, skip: usize) -> bool {
        let content1 = read_file_without_first_lines(file1, skip);
        let content2 = read_file_without_first_lines(file2, skip);
        content1 == content2
    }

    fn read_file_without_first_lines(file: &str, skip: usize) -> Vec<String> {
        let reader = BufReader::new(File::open(file).unwrap());
        reader
            .lines()
            .skip(skip)
            .map(|line| line.unwrap())
            .collect()
    }

    #[test]
    fn import_and_run() {
        let _ = std::fs::create_dir("temporary");

        let mut app = GuiOrderApp::default();
        app.import_yaml("tests/parameters.yaml");
        app.run_analysis();

        // wait for the completion of the analysis
        while *app.running.lock().unwrap() {}

        assert_eq_order("temporary/order.yaml", "tests/output/order.yaml", 1);
        assert_eq_order("temporary/order.tab", "tests/output/order.tab", 1);
        assert_eq_order("temporary/order_POPC.xvg", "tests/output/order_POPC.xvg", 1);
        assert_eq_order(
            "temporary/convergence.xvg",
            "tests/output/convergence.xvg",
            1,
        );
        assert_eq_csv("temporary/order.csv", "tests/output/order.csv", 0);

        assert_eq_maps(
            "temporary/ordermaps/ordermap_average_full.dat",
            "tests/output/ordermaps/ordermap_average_full.dat",
            2,
        );

        assert_eq_maps(
            "temporary/ordermaps/ordermap_average_lower.dat",
            "tests/output/ordermaps/ordermap_average_lower.dat",
            2,
        );

        assert_eq_maps(
            "temporary/ordermaps/ordermap_average_upper.dat",
            "tests/output/ordermaps/ordermap_average_upper.dat",
            2,
        );

        let maps = [
            "ordermap_average_full.dat",
            "ordermap_POPC-C210-64--POPC-H101-65_full.dat",
            "ordermap_POPC-C215-78_lower.dat",
            "ordermap_POPC-C215-78--POPC-H15S-80_lower.dat",
            "ordermap_average_lower.dat",
            "ordermap_POPC-C210-64--POPC-H101-65_lower.dat",
            "ordermap_POPC-C215-78--POPC-H15R-79_full.dat",
            "ordermap_POPC-C215-78--POPC-H15S-80_upper.dat",
            "ordermap_average_upper.dat",
            "ordermap_POPC-C210-64--POPC-H101-65_upper.dat",
            "ordermap_POPC-C215-78--POPC-H15R-79_lower.dat",
            "ordermap_POPC-C215-78_upper.dat",
            "ordermap_POPC-C210-64_full.dat",
            "ordermap_POPC-C210-64_upper.dat",
            "ordermap_POPC-C215-78--POPC-H15R-79_upper.dat",
            "ordermap_POPC-C210-64_lower.dat",
            "ordermap_POPC-C215-78_full.dat",
            "ordermap_POPC-C215-78--POPC-H15S-80_full.dat",
        ];

        for map in maps {
            assert_eq_maps(
                &format!("temporary/ordermaps/POPC/{}", map),
                &format!("tests/output/ordermaps/POPC/{}", map),
                2,
            )
        }

        std::fs::remove_dir_all("temporary").unwrap();
    }

    #[test]
    fn import_and_export() {
        let output = NamedTempFile::new().unwrap();
        let path_to_output = output.path().to_path_buf();

        let mut app = GuiOrderApp::default();
        app.import_yaml("tests/parameters.yaml");
        app.export_to_yaml(path_to_output.clone());

        assert!(diff_files_ignore_first(
            path_to_output.to_str().unwrap(),
            "tests/exported.yaml",
            1
        ));
    }
}
