// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Parameters for frame selection.

use eframe::egui::{DragValue, Ui};

use crate::GuiAnalysis;

/// Parameters for frame selection.
#[derive(Debug, Clone)]
pub(crate) struct FrameSelectionParams {
    pub begin: f32,
    pub end: f32,
    pub step: usize,
}

impl FrameSelectionParams {
    pub(crate) fn new(begin: f32, end: f32, step: usize) -> Self {
        Self { begin, end, step }
    }
}

impl Default for FrameSelectionParams {
    fn default() -> Self {
        Self {
            begin: 0.0,
            end: f32::INFINITY,
            step: 1,
        }
    }
}

fn format_with_commas<T: std::fmt::Display>(num: T) -> String {
    let s = num.to_string();
    let parts: Vec<&str> = s.split('.').collect();
    let integer = parts[0];
    let decimal = if parts.len() > 1 { parts[1] } else { "" };

    let mut result = String::new();
    let mut count = 0;

    for c in integer.chars().rev() {
        if count != 0 && count % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
        count += 1;
    }

    let formatted_integer = result.chars().rev().collect::<String>();

    if decimal.is_empty() {
        formatted_integer
    } else {
        format!("{}.{}", formatted_integer, decimal)
    }
}

impl GuiAnalysis {
    /// Specify the parameters for the frame selection.
    pub(super) fn specify_frame_selection(&mut self, ui: &mut Ui) {
        Self::collapsing_with_warning(ui, "Frame selection", false, true, |ui| {
            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Begin: ",
                    "Start to read the trajectory from this time.",
                );

                ui.add(
                    DragValue::new(&mut self.frame_selection_params.begin)
                        .speed(200)
                        .range(0.0..=self.frame_selection_params.end)
                        .suffix(" ps")
                        .custom_formatter(|n, _| format_with_commas(n)),
                );

                Self::label_with_hint(
                    ui,
                    "   End: ",
                    "Finish reading the trajectory at this time.",
                );

                let response = ui.add(
                    DragValue::new(&mut self.frame_selection_params.end)
                        .speed(200)
                        .range(self.frame_selection_params.begin..=f32::INFINITY)
                        .suffix(" ps")
                        .custom_formatter(|n, _| format_with_commas(n)),
                );

                // makes it possible to decrease from infinity
                if self.frame_selection_params.end.is_infinite() {
                    if response.dragged() {
                        self.frame_selection_params.end =
                            self.frame_selection_params.begin + 100_000.0;
                    }
                }

                Self::label_with_hint(ui, "   Step: ", "Read every Nth frame.");
                ui.add(
                    DragValue::new(&mut self.frame_selection_params.step)
                        .speed(0.1)
                        .range(1..=usize::MAX),
                );
            });
        });
    }
}
