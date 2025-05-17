// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Handles spawning windows.

use std::collections::HashMap;

use eframe::egui::{self, Id, RichText, Ui};
use regex::Regex;

use crate::GuiOrderApp;

/// A single window.
#[derive(Debug, Clone)]
struct Window {
    title: String,
    messages: Vec<Message>,
    open: bool,
}

/// Single message to print to a window.
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Message {
    Error(String),
    Warning(String),
    Info(String),
}

impl Message {
    fn write(&self, ui: &mut Ui) {
        let color = match self {
            Self::Error(_) => egui::Color32::from_rgba_premultiplied(150, 0, 0, 100),
            Self::Warning(_) => egui::Color32::from_rgba_premultiplied(150, 120, 0, 100),
            Self::Info(_) => egui::Color32::from_rgba_premultiplied(150, 150, 150, 100),
        };

        let text = match self {
            Self::Error(x) | Self::Warning(x) | Self::Info(x) => x,
        };

        let label = match self {
            Self::Error(_) => "error:",
            Self::Warning(_) => "warning: ",
            Self::Info(_) => "",
        };

        ui.label(
            RichText::new(Message::insert_newlines(
                &format!(
                    "{} {}",
                    label,
                    Message::remove_error_labels(&Message::strip_ansi_codes(text))
                ),
                60,
            ))
            .font(egui::FontId::monospace(12.0))
            .color(color),
        );
    }

    /// Remove ANSI codes from a string.
    fn strip_ansi_codes(input: &str) -> String {
        let re = Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
        re.replace_all(input, "").into_owned()
    }

    /// Remove all `error:` labels from the string.
    fn remove_error_labels(input: &str) -> String {
        input.replace("error: ", "")
    }

    /// Format string to fit into a rectangle of specific width.
    fn insert_newlines(input: &str, n: usize) -> String {
        input
            .chars()
            .fold((String::new(), 1), |(mut acc, mut count), c| {
                if c == '\n' {
                    acc.push_str(&format!("\n | "));
                    count = 1;
                } else {
                    if count % n == 0 {
                        acc.push('\n');
                    }
                    acc.push(c);
                    count += 1;
                }
                (acc, count)
            })
            .0
    }
}

/// All spawned error windows.
#[derive(Debug, Clone, Default)]
pub(crate) struct Windows {
    windows: HashMap<Id, Window>,
    total_spawned: usize,
}

impl Window {
    /// Render the window.
    fn render(&mut self, id: Id, ctx: &egui::Context) {
        egui::Window::new(&self.title)
            .id(id)
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .default_width(600.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for message in &self.messages {
                        message.write(ui);
                    }
                });
            });
    }
}

impl Windows {
    /// Render all windows.
    pub(super) fn render(&mut self, ctx: &egui::Context) {
        let mut windows_to_close = vec![];
        for (id, window) in self.windows.iter_mut() {
            if !window.open {
                windows_to_close.push(id.clone());
                continue;
            }

            window.render(*id, ctx);
        }

        // clean up closed windows
        for id in windows_to_close {
            self.windows.remove(&id);
        }
    }
}

impl GuiOrderApp {
    /// Open a new error window.
    pub(super) fn open_error_window(&mut self, error: Box<dyn std::error::Error + Send + Sync>) {
        self.windows.windows.insert(
            Id::new(self.windows.total_spawned),
            Window {
                title: String::from("Error!"),
                messages: vec![Message::Error(error.to_string())],
                open: true,
            },
        );

        self.windows.total_spawned += 1;
    }
}
