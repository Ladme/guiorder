// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Parameters for error estimation.

use eframe::egui::{DragValue, Ui};

use crate::{error::ConversionError, GuiAnalysis};

#[derive(Debug, Clone)]
/// Parameters for the error estimation.
pub(crate) struct EstimateErrorParams {
    estimate_error: bool,
    n_blocks: usize,
    output_convergence: String,
}

impl Default for EstimateErrorParams {
    fn default() -> Self {
        Self {
            estimate_error: false,
            n_blocks: 5,
            output_convergence: String::new(),
        }
    }
}

impl From<Option<gorder::input::EstimateError>> for EstimateErrorParams {
    fn from(value: Option<gorder::input::EstimateError>) -> Self {
        match value {
            None => Self {
                estimate_error: false,
                ..Default::default()
            },
            Some(x) => Self {
                estimate_error: true,
                n_blocks: x.n_blocks(),
                output_convergence: match x.output_convergence().clone() {
                    None => String::new(),
                    Some(x) => x.to_string(),
                },
            },
        }
    }
}

impl TryFrom<&EstimateErrorParams> for Option<gorder::input::EstimateError> {
    type Error = ConversionError;
    fn try_from(value: &EstimateErrorParams) -> Result<Self, Self::Error> {
        if !value.estimate_error {
            return Ok(None);
        }

        let convergence = match value.output_convergence.is_empty() {
            false => Some(value.output_convergence.as_str()),
            true => None,
        };

        Ok(Some(
            gorder::input::EstimateError::new(Some(value.n_blocks), convergence)
                .map_err(|e| ConversionError::InvalidEstimateError(e.to_string()))?,
        ))
    }
}

impl GuiAnalysis {
    pub(super) fn specify_estimate_error(&mut self, ui: &mut Ui) {
        Self::collapsing_with_warning(ui, "Error estimation", false, true, |ui| {
            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Estimate error: ",
                    "Check the box if you want to estimate the analysis error using block averaging.",
                );
                ui.checkbox(&mut self.estimate_error_params.estimate_error, "");
            });

            if !self.estimate_error_params.estimate_error {
                return;
            }

            ui.horizontal(|ui| {
                Self::label_with_hint(
                    ui,
                    "Blocks:      ",
                    "Number of blocks to use for block averaging.",
                );

                ui.add(
                    DragValue::new(&mut self.estimate_error_params.n_blocks)
                        .speed(0.1)
                        .range(2..=usize::MAX),
                );
            });

            Self::specify_output_file(
                &mut self.estimate_error_params.output_convergence,
                ui,
                "Convergence: ",
                "Path to an output XVG file where the convergence of the analyzed simulation will be written. (Optional.)",
                false
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_estimate_error() {
        let converted = EstimateErrorParams::from(Some(
            gorder::input::EstimateError::new(Some(10), Some("convergence.yaml")).unwrap(),
        ));

        assert!(converted.estimate_error);
        assert_eq!(converted.n_blocks, 10);
        assert_eq!(
            converted.output_convergence,
            String::from("convergence.yaml")
        );

        let converted = EstimateErrorParams::from(None);

        assert!(!converted.estimate_error);
    }
}
