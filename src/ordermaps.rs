// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Parameters for constructing ordermaps.

use eframe::egui::{self, RichText, Ui};

use crate::{common::MembraneNormal, error::ConversionError, GuiAnalysis};

/// How are ordermap dimensions set?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum OrderMapDimension {
    #[default]
    Auto,
    Manual,
}

impl From<gorder::input::GridSpan> for OrderMapDimension {
    fn from(value: gorder::input::GridSpan) -> Self {
        match value {
            gorder::input::GridSpan::Auto => OrderMapDimension::Auto,
            gorder::input::GridSpan::Manual { .. } => OrderMapDimension::Manual,
        }
    }
}

/// Plane in which the ordermaps shall be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Plane {
    Unknown,
    #[default]
    XY,
    YZ,
    XZ,
}

/// Parameters for the manual dimensions specification.
#[derive(Debug, Clone)]
struct ManualDimensions {
    start: f32,
    end: f32,
}

impl From<gorder::input::GridSpan> for ManualDimensions {
    fn from(value: gorder::input::GridSpan) -> Self {
        match value {
            gorder::input::GridSpan::Auto => ManualDimensions::default(),
            gorder::input::GridSpan::Manual { start, end } => ManualDimensions { start, end },
        }
    }
}

impl Default for ManualDimensions {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 10.0,
        }
    }
}

/// Parameters for the construction of ordermaps.
#[derive(Debug, Clone)]
pub(crate) struct OrderMapsParams {
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

impl From<Option<gorder::input::OrderMap>> for OrderMapsParams {
    fn from(value: Option<gorder::input::OrderMap>) -> Self {
        match value {
            None => Self {
                calculate_maps: false,
                ..Default::default()
            },
            Some(map) => Self {
                calculate_maps: true,
                output_directory: map.output_directory().clone().unwrap_or(String::new()),
                plane: match map.plane() {
                    None => None,
                    Some(gorder::input::Plane::XY) => Some(Plane::XY),
                    Some(gorder::input::Plane::XZ) => Some(Plane::XZ),
                    Some(gorder::input::Plane::YZ) => Some(Plane::YZ),
                },
                bin_size: map.bin_size(),
                dimensions: [map.dim()[0].clone().into(), map.dim()[1].clone().into()],
                x_manual: map.dim()[0].clone().into(),
                y_manual: map.dim()[1].clone().into(),
                min_samples: map.min_samples(),
            },
        }
    }
}

impl TryFrom<&OrderMapsParams> for Option<gorder::input::OrderMap> {
    type Error = ConversionError;
    fn try_from(value: &OrderMapsParams) -> Result<Self, Self::Error> {
        if !value.calculate_maps {
            return Ok(None);
        }

        let dimension_x = match value.dimensions[0] {
            OrderMapDimension::Auto => gorder::input::GridSpan::Auto,
            OrderMapDimension::Manual => {
                gorder::input::GridSpan::manual(value.x_manual.start, value.x_manual.end).unwrap()
            }
        };

        let dimension_y = match value.dimensions[1] {
            OrderMapDimension::Auto => gorder::input::GridSpan::Auto,
            OrderMapDimension::Manual => {
                gorder::input::GridSpan::manual(value.y_manual.start, value.y_manual.end).unwrap()
            }
        };

        let mut builder = gorder::input::OrderMap::builder();

        builder
            .output_directory(&value.output_directory)
            .bin_size(value.bin_size)
            .dim([dimension_x, dimension_y])
            .min_samples(value.min_samples);

        if let Some(plane) = value.plane {
            let converted_plane = match plane {
                Plane::XY => gorder::input::Plane::XY,
                Plane::XZ => gorder::input::Plane::XZ,
                Plane::YZ => gorder::input::Plane::YZ,
                Plane::Unknown => {
                    return Err(ConversionError::InvalidOrderMapParams(String::from(
                        "unknown plane",
                    )))
                }
            };

            builder.plane(converted_plane);
        }

        let ordermap = builder
            .build()
            .map_err(|e| ConversionError::InvalidOrderMapParams(e.to_string()))?;

        Ok(Some(ordermap))
    }
}

impl GuiAnalysis {
    /// Specify parameters for the construction of ordermaps.
    pub(super) fn specify_ordermaps(&mut self, ui: &mut Ui) {
        Self::collapsing_with_warning(
            ui,
            "Order parameter maps",
            false,
            self.check_ordermaps_sanity(),
            |ui| {
                ui.horizontal(|ui| {
                    Self::label_with_hint(
                        ui,
                        "Construct ordermaps: ",
                        "Check the box if you want the ordermaps to be constructed.",
                    );
                    ui.checkbox(&mut self.ordermaps_params.calculate_maps, "");
                });

                if !self.ordermaps_params.calculate_maps {
                    return;
                }

                Self::specify_string(
                &mut self.ordermaps_params.output_directory,
                ui,
                "Directory:    ", 
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
                        MembraneNormal::Dynamic | MembraneNormal::FromFile => Plane::Unknown,
                    }
                };

                // specify plane of the maps
                ui.horizontal(|ui| {
                    Self::label_with_hint(
                        ui,
                        "Plane:        ",
                        "Plane in which the ordermaps will be constructed.",
                    );

                    if ui
                        .add(egui::RadioButton::new(raw_plane == Plane::XY, "xy"))
                        .clicked()
                    {
                        self.ordermaps_params.plane = Some(Plane::XY);
                    }

                    if ui
                        .add(egui::RadioButton::new(raw_plane == Plane::XZ, "xz"))
                        .clicked()
                    {
                        self.ordermaps_params.plane = Some(Plane::XZ);
                    }

                    if ui
                        .add(egui::RadioButton::new(raw_plane == Plane::YZ, "yz"))
                        .clicked()
                    {
                        self.ordermaps_params.plane = Some(Plane::YZ);
                    }

                    if raw_plane == Plane::Unknown {
                        ui.label(
                            RichText::new("❗")
                                .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)),
                        );
                    }
                });

                let (dim_1, dim_2) = match raw_plane {
                    Plane::XY => ("X-dimension", "Y-dimension"),
                    Plane::YZ => ("Z-dimension", "Y-dimension"),
                    Plane::XZ => ("X-dimension", "Z-dimension"),
                    Plane::Unknown => ("Unknown dimension", "Unknown dimension"),
                };

                // specify size of the maps
                ui.vertical(|ui| {
                    Self::label_with_hint(ui, "Maps size: ", "Size of the ordermaps.");

                    Self::specify_dimension(
                        &mut self.ordermaps_params.dimensions[0],
                        &mut self.ordermaps_params.x_manual.start,
                        &mut self.ordermaps_params.x_manual.end,
                        ui,
                        dim_1,
                    );

                    Self::specify_dimension(
                        &mut self.ordermaps_params.dimensions[1],
                        &mut self.ordermaps_params.y_manual.start,
                        &mut self.ordermaps_params.y_manual.end,
                        ui,
                        dim_2,
                    );
                });

                // specify bin size
                ui.horizontal(|ui| {
                    Self::label_with_hint(
                        ui,
                        "Bin size:     ",
                        "Size of the bins of the ordermap.",
                    );
                    Self::specify_bin_size(&mut self.ordermaps_params.bin_size[0], ui, dim_1);
                    Self::specify_bin_size(&mut self.ordermaps_params.bin_size[1], ui, dim_2);
                });

                // specify minimum number of samples per bin
                ui.horizontal(|ui| {
                    Self::label_with_hint(
                        ui,
                        "Min samples:  ",
                        "Minimum number of samples required in a bin to calculate order parameter.",
                    );

                    ui.add(
                        egui::DragValue::new(&mut self.ordermaps_params.min_samples)
                            .speed(2.5)
                            .range(1..=usize::MAX),
                    );
                });
            },
        );
    }

    /// Specify the size of the map in a particular dimension.
    fn specify_dimension(
        dim: &mut OrderMapDimension,
        dim_start: &mut f32,
        dim_end: &mut f32,
        ui: &mut Ui,
        dim_label: &str,
    ) {
        ui.horizontal(|ui| {
            Self::label_with_hint(
                ui,
                &format!(" {dim_label}: "),
                &format!(
                    "Size of the ordermaps along the {}.",
                    dim_label.to_lowercase()
                ),
            );

            ui.radio_value(dim, OrderMapDimension::Auto, "automatic");
            ui.radio_value(dim, OrderMapDimension::Manual, "manual");

            if *dim == OrderMapDimension::Manual {
                ui.add(
                    egui::DragValue::new(dim_start)
                        .speed(0.1)
                        .range(-f32::MAX..=*dim_end)
                        .suffix(" nm"),
                )
                .on_hover_ui(|ui| {
                    ui.label("start");
                });

                ui.add(
                    egui::DragValue::new(dim_end)
                        .speed(0.1)
                        .range(*dim_start..=f32::MAX)
                        .suffix(" nm"),
                )
                .on_hover_ui(|ui| {
                    ui.label("end");
                });
            }
        });
    }

    /// Specify the size of a bin in a particular dimension.
    fn specify_bin_size(bin_size: &mut f32, ui: &mut Ui, dim_label: &str) {
        ui.add(
            egui::DragValue::new(bin_size)
                .speed(0.01)
                .range(0.0..=f32::MAX)
                .suffix(" nm"),
        )
        .on_hover_ui(|ui| {
            ui.label(dim_label.to_lowercase());
        });

        if *bin_size == 0.0 {
            ui.label(
                RichText::new("❗").color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)),
            );
        } else {
            ui.add_space(21.0);
        }
    }

    /// Check that all parameters for the construction of ordermaps have been provided.
    pub(super) fn check_ordermaps_sanity(&self) -> bool {
        !self.ordermaps_params.calculate_maps
            || (!self.ordermaps_params.output_directory.is_empty()
                && (self.ordermaps_params.plane.is_some()
                    || self.membrane_normal != MembraneNormal::Dynamic)
                && self.ordermaps_params.bin_size[0] > 0.0
                && self.ordermaps_params.bin_size[1] > 0.0)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn convert_grid_span() {
        assert_eq!(
            OrderMapDimension::from(gorder::input::GridSpan::Auto),
            OrderMapDimension::Auto
        );
        assert_eq!(
            OrderMapDimension::from(gorder::input::GridSpan::manual(2.0, 5.0).unwrap()),
            OrderMapDimension::Manual
        );
    }

    #[test]
    fn convert_manual_dimensions() {
        let params = ManualDimensions::from(gorder::input::GridSpan::manual(2.0, 5.0).unwrap());

        assert_relative_eq!(params.start, 2.0);
        assert_relative_eq!(params.end, 5.0);
    }

    #[test]
    fn convert_ordermaps_params() {
        let params = OrderMapsParams::try_from(None).unwrap();
        assert!(!params.calculate_maps);

        let params = OrderMapsParams::try_from(Some(
            gorder::input::OrderMap::builder()
                .bin_size([0.05, 0.2])
                .dim([
                    gorder::input::GridSpan::Auto,
                    gorder::input::GridSpan::manual(-3.0, 10.0).unwrap(),
                ])
                .min_samples(100)
                .plane(gorder::input::Plane::XZ)
                .output_directory("ordermaps")
                .build()
                .unwrap(),
        ))
        .unwrap();

        assert!(params.calculate_maps);
        assert_relative_eq!(params.bin_size[0], 0.05);
        assert_relative_eq!(params.bin_size[1], 0.2);
        assert_eq!(params.dimensions[0], OrderMapDimension::Auto);
        assert_eq!(params.dimensions[1], OrderMapDimension::Manual);
        assert_relative_eq!(params.y_manual.start, -3.0);
        assert_relative_eq!(params.y_manual.end, 10.0);
        assert_eq!(params.min_samples, 100);
        assert_eq!(params.plane, Some(Plane::XZ));
        assert_eq!(params.output_directory, String::from("ordermaps"));
    }
}
