// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Errors associated with guiorder.

use colored::Colorize;
use thiserror::Error;

/// Errors returned when attempting conversion between gorder's Analysis and GuiAnalysis structures.
#[derive(Debug, Clone, Error)]
pub enum ConversionError {
    #[error("{} guiorder does not support inline specification of membrane normals", "error:".red().bold())]
    FromMapNormals,
    #[error("{} guiorder does not support inline specification of leaflet assignment", "error:".red().bold())]
    FromMapLeaflets,
    #[error("{} could not convert ordermap parameters into the gorder structure (details: {})", "error:".red().bold(), .0.yellow())]
    InvalidOrderMapParams(String),
    #[error("{} could not convert analysis parameters into the gorder structure (details: {})", "error:".red().bold(), .0.yellow())]
    InvalidAnalysisParams(String),
}
