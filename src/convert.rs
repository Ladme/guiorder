// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Conversion from and to gorder::Analysis.

use gorder::input::Analysis;

use crate::{
    error::ConversionError, frame_selection::FrameSelectionParams, window::Windows, GuiAnalysis,
};

impl TryFrom<Analysis> for GuiAnalysis {
    type Error = ConversionError;
    fn try_from(value: Analysis) -> Result<Self, Self::Error> {
        Ok(Self {
            structure: value.structure().clone(),
            trajectory: value.trajectory().clone(),
            ndx: value.index().clone().unwrap_or(String::new()),
            bonds: value.bonds().clone().unwrap_or(String::new()),
            output: (&value).into(),
            analysis_type: value.analysis_type().clone().into(),
            analysis_type_params: value.analysis_type().clone().into(),
            membrane_normal: value.membrane_normal().clone().try_into()?,
            dynamic_normal_params: value.membrane_normal().clone().try_into()?,
            from_file_normals: if let gorder::input::MembraneNormal::FromFile(x) =
                value.membrane_normal()
            {
                x.clone()
            } else {
                String::new()
            },
            leaflet_classification_method: value.leaflets().clone().try_into()?,
            leaflet_classification_params: value.leaflets().clone().try_into()?,
            estimate_error_params: value.estimate_error().clone().into(),
            frame_selection_params: FrameSelectionParams::new(
                value.begin(),
                value.end(),
                value.step(),
            ),
            geom_selection: value.geometry().clone().into(),
            geom_selection_params: value.geometry().clone().into(),
            ordermaps_params: value.map().clone().into(),
            other_params: (&value).into(),
            windows: Windows::default(),
        })
    }
}
