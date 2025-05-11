// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Handles spawning windows.

use std::rc::Rc;

use eframe::egui::{self, Id, RichText};

use crate::GuiAnalysis;

/// All spawned error windows.
#[derive(Debug, Clone, Default)]
pub(crate) struct Windows {
    windows: Vec<bool>,
    errors: Vec<Rc<dyn std::error::Error>>,
}

impl Windows {
    /// Render all windows.
    pub(super) fn render(&mut self, ctx: &egui::Context) {
        let mut windows_to_close = vec![];
        for (id, open) in self.windows.iter_mut().enumerate() {
            if !*open {
                windows_to_close.push(id.clone());
                continue;
            }

            // render windows
            egui::Window::new("Error!")
                .id(Id::new(id))
                .open(open)
                .collapsible(false)
                .resizable(false)
                .default_width(100.0)
                .default_height(100.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(GuiAnalysis::insert_newlines(
                                &GuiAnalysis::strip_ansi_codes(
                                    &self.errors.get(id).unwrap().to_string(),
                                ),
                                60,
                            ))
                            .font(egui::FontId::monospace(12.0))
                            .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)),
                        );
                    });
                });
        }

        // clean up closed windows
        for id in windows_to_close {
            self.windows.remove(id);
            self.errors.remove(id);
        }
    }
}

impl GuiAnalysis {
    /// Open a new error window.
    pub(super) fn open_window(&mut self, error: Rc<dyn std::error::Error>) {
        self.windows.windows.push(true);
        self.windows.errors.push(error);
    }
}
