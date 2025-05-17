// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Parameters for geometric selection.

use std::fmt::Display;

use eframe::egui::{self, ComboBox, DragValue, Response, RichText, Ui};
use gorder::{input::Axis, prelude::Vector3D};

use crate::GuiAnalysis;

/// Geometric selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum GeomSelection {
    #[default]
    None,
    Cuboid,
    Cylinder,
    Sphere,
}

impl From<Option<gorder::input::Geometry>> for GeomSelection {
    fn from(value: Option<gorder::input::Geometry>) -> Self {
        match value {
            None => GeomSelection::None,
            Some(gorder::input::Geometry::Cuboid(_)) => GeomSelection::Cuboid,
            Some(gorder::input::Geometry::Cylinder(_)) => GeomSelection::Cylinder,
            Some(gorder::input::Geometry::Sphere(_)) => GeomSelection::Sphere,
        }
    }
}

/// Parameters for geometric selection.
#[derive(Debug, Clone, Default)]
pub(crate) struct GeomSelectionParams {
    cuboid: CuboidParams,
    cylinder: CylinderParams,
    sphere: SphereParams,
    reference_type: GeomReferenceType,
    ref_point: Vector3D,
    ref_selection: String,
}

impl From<Option<gorder::input::Geometry>> for GeomSelectionParams {
    fn from(value: Option<gorder::input::Geometry>) -> Self {
        match value {
            None => Self::default(),
            Some(gorder::input::Geometry::Cuboid(params)) => Self {
                cuboid: CuboidParams {
                    minx: params.xdim()[0],
                    maxx: params.xdim()[1],
                    miny: params.ydim()[0],
                    maxy: params.ydim()[1],
                    minz: params.zdim()[0],
                    maxz: params.zdim()[1],
                },
                reference_type: params.reference().clone().into(),
                ref_point: get_static_reference_point(params.reference()),
                ref_selection: get_reference_selection(params.reference()),
                ..Default::default()
            },
            Some(gorder::input::Geometry::Cylinder(params)) => Self {
                cylinder: CylinderParams {
                    radius: params.radius(),
                    start: params.span()[0],
                    end: params.span()[1],
                    orientation: params.orientation(),
                },
                reference_type: params.reference().clone().into(),
                ref_point: get_static_reference_point(params.reference()),
                ref_selection: get_reference_selection(params.reference()),
                ..Default::default()
            },
            Some(gorder::input::Geometry::Sphere(params)) => Self {
                sphere: SphereParams {
                    radius: params.radius(),
                },
                reference_type: params.reference().clone().into(),
                ref_point: get_static_reference_point(params.reference()),
                ref_selection: get_reference_selection(params.reference()),
                ..Default::default()
            },
        }
    }
}

impl From<gorder::input::GeomReference> for GeomReferenceType {
    fn from(value: gorder::input::GeomReference) -> Self {
        match value {
            gorder::input::GeomReference::Center => GeomReferenceType::Center,
            gorder::input::GeomReference::Point(_) => GeomReferenceType::Point,
            gorder::input::GeomReference::Selection(_) => GeomReferenceType::Selection,
        }
    }
}

fn get_static_reference_point(reference: &gorder::input::GeomReference) -> Vector3D {
    match reference {
        gorder::input::GeomReference::Point(x) => x.clone(),
        _ => Vector3D::default(),
    }
}

fn get_reference_selection(reference: &gorder::input::GeomReference) -> String {
    match reference {
        gorder::input::GeomReference::Selection(x) => x.clone(),
        _ => String::new(),
    }
}

impl GeomSelectionParams {
    /// Allows drag value to get changed from infinity.
    fn change_from_infinity(response: &Response, value: &mut f32, target: f32) {
        if value.is_infinite() {
            if response.dragged() {
                *value = target;
            }
        }
    }

    /// Specify span along a particular dimension.
    fn specify_span(ui: &mut Ui, start: &mut f32, end: &mut f32) {
        let start_response = ui
            .add(
                egui::DragValue::new(start)
                    .speed(0.05)
                    .range(f32::NEG_INFINITY..=*end)
                    .suffix(" nm"),
            )
            .on_hover_ui(|ui| {
                ui.label("start");
            });

        let end_response = ui
            .add(
                egui::DragValue::new(end)
                    .speed(0.05)
                    .range(*start..=f32::INFINITY)
                    .suffix(" nm"),
            )
            .on_hover_ui(|ui| {
                ui.label("end");
            });

        Self::change_from_infinity(&start_response, start, 0.0);
        if start.is_finite() {
            Self::change_from_infinity(&end_response, end, *start);
        } else {
            Self::change_from_infinity(&end_response, end, 0.0);
        }
    }

    /// Specify the reference point.
    fn specify_reference(&mut self, ui: &mut Ui, label: &str, hint: &str) {
        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(ui, label, hint);

            ui.radio_value(&mut self.reference_type, GeomReferenceType::Point, "point")
                .on_hover_ui(|ui| {
                    ui.label("Static, unscaled point.");
                });
            ui.radio_value(
                &mut self.reference_type,
                GeomReferenceType::Center,
                "box center",
            )
            .on_hover_ui(|ui| {
                ui.label("Dynamically calculated simulation box center.");
            });
            ui.radio_value(
                &mut self.reference_type,
                GeomReferenceType::Selection,
                "selection center",
            )
            .on_hover_ui(|ui| {
                ui.label(
                    "Dynamically calculated center of geometry of the provided atom selection.",
                );
            });
        });

        match self.reference_type {
            GeomReferenceType::Point => self.specify_static_point(ui),
            GeomReferenceType::Center => (),
            GeomReferenceType::Selection => {
                ui.horizontal(|ui| {
                    GuiAnalysis::specify_string(
                    &mut self.ref_selection,
                    ui,
                    " Selection:  ",
                    "Selection of atoms which center of geometry represents the reference point.",
                    true,
                );
                });
            }
        }
    }

    /// Specify a point in 3D space.
    fn specify_static_point(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(ui, " Coordinates:  ", "Coordinates of the point.");

            for (i, hint) in ["x-coordinate", "y-coordinate", "z-coordinate"]
                .into_iter()
                .enumerate()
            {
                ui.add(
                    egui::DragValue::new(&mut self.ref_point[i])
                        .speed(0.05)
                        .range(-f32::MAX..=f32::MAX)
                        .suffix(" nm"),
                )
                .on_hover_ui(|ui| {
                    ui.label(hint);
                });
            }
        });
    }
}

/// Parameters for cuboidal selection.
#[derive(Debug, Clone)]
struct CuboidParams {
    minx: f32,
    maxx: f32,
    miny: f32,
    maxy: f32,
    minz: f32,
    maxz: f32,
}

impl Default for CuboidParams {
    fn default() -> Self {
        Self {
            minx: f32::NEG_INFINITY,
            maxx: f32::INFINITY,
            miny: f32::NEG_INFINITY,
            maxy: f32::INFINITY,
            minz: f32::NEG_INFINITY,
            maxz: f32::INFINITY,
        }
    }
}

impl CuboidParams {
    /// Specify parameters for cuboidal selection.
    fn specify(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(
                ui,
                "X-dimension: ",
                "Span of the cuboid along the x-dimension.",
            );

            GeomSelectionParams::specify_span(ui, &mut self.minx, &mut self.maxx);
        });

        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(
                ui,
                "Y-dimension: ",
                "Span of the cuboid along the y-dimension.",
            );

            GeomSelectionParams::specify_span(ui, &mut self.miny, &mut self.maxy);
        });

        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(
                ui,
                "Z-dimension: ",
                "Span of the cuboid along the z-dimension.",
            );

            GeomSelectionParams::specify_span(ui, &mut self.minz, &mut self.maxz);
        });
    }
}

/// Parameters for cylindrical selection.
#[derive(Debug, Clone)]
struct CylinderParams {
    radius: f32,
    start: f32,
    end: f32,
    orientation: Axis,
}

impl Default for CylinderParams {
    fn default() -> Self {
        Self {
            radius: 5.0,
            start: f32::NEG_INFINITY,
            end: f32::INFINITY,
            orientation: Axis::Z,
        }
    }
}

impl CylinderParams {
    /// Specify parameters for cylindrical selection.
    fn specify(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(ui, "Radius:   ", "Radius of the cylinder.");

            ui.add(
                DragValue::new(&mut self.radius)
                    .speed(0.05)
                    .range(0.0..=f32::INFINITY)
                    .suffix(" nm"),
            );

            if self.radius == 0.0 {
                ui.label(
                    RichText::new("❗")
                        .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)),
                );
            }
        });

        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(
                ui,
                "Span:     ",
                "Span of the cylinder along its main axis.",
            );

            GeomSelectionParams::specify_span(ui, &mut self.start, &mut self.end);
        });

        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(
                ui,
                "Orientation: ",
                "Orientation of the main axis of the cylinder.",
            );

            ui.radio_value(&mut self.orientation, Axis::X, "x");
            ui.radio_value(&mut self.orientation, Axis::Y, "y");
            ui.radio_value(&mut self.orientation, Axis::Z, "z");
        });
    }

    /// Check that all parameters have been provided.
    fn sanity_check(&self) -> bool {
        self.radius > 0.0
    }
}

/// Parameters for spherical selection.
#[derive(Debug, Clone)]
struct SphereParams {
    radius: f32,
}

impl Default for SphereParams {
    fn default() -> Self {
        Self { radius: 5.0 }
    }
}

impl SphereParams {
    /// Specify parameters for spherical selection.
    fn specify(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            GuiAnalysis::label_with_hint(ui, "Radius:   ", "Radius of the sphere.");

            ui.add(
                DragValue::new(&mut self.radius)
                    .speed(0.05)
                    .range(0.0..=f32::INFINITY)
                    .suffix(" nm"),
            );

            if self.radius == 0.0 {
                ui.label(
                    RichText::new("❗")
                        .color(egui::Color32::from_rgba_premultiplied(150, 0, 0, 100)),
                );
            }
        });
    }

    /// Check that all parameters have been provided.
    fn sanity_check(&self) -> bool {
        self.radius > 0.0
    }
}

/// Type of reference for the geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum GeomReferenceType {
    #[default]
    Point,
    Selection,
    Center,
}

impl Display for GeomSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeomSelection::None => write!(f, ""),
            GeomSelection::Cuboid => write!(f, "cuboid"),
            GeomSelection::Cylinder => write!(f, "cylinder"),
            GeomSelection::Sphere => write!(f, "sphere"),
        }
    }
}

impl GuiAnalysis {
    /// Specify the parameters for geometric selection.
    pub(super) fn specify_geometry(&mut self, ui: &mut Ui) {
        Self::collapsing_with_warning(
            ui,
            "Region selection",
            false,
            self.check_geometry_sanity(),
            |ui| {
                ui.horizontal(|ui| {
                    Self::label_with_hint(
                        ui,
                        "Geometry: ",
                        "Geometry specifying the membrane region in which order parameters are to be calculated.",
                    );

                    ComboBox::from_label("")
                        .selected_text(format!("{}", self.geom_selection))
                        .show_ui(ui, |ui| {
                            for variant in [
                                GeomSelection::None,
                                GeomSelection::Cuboid,
                                GeomSelection::Cylinder,
                                GeomSelection::Sphere,
                            ] {
                                ui.selectable_value(
                                    &mut self.geom_selection,
                                    variant.clone(),
                                    format!("{}", variant),
                                );
                            }
                        });

                    if let GeomSelection::None = self.geom_selection {
                        ui.label(
                            RichText::new("whole system selected")
                                .font(egui::FontId::proportional(10.0)),
                        );
                    }
                    ui.end_row();
                });

                match self.geom_selection {
                    GeomSelection::None => (),
                    GeomSelection::Cuboid => {
                        self.geom_selection_params.cuboid.specify(ui);
                        self.geom_selection_params.specify_reference(
                            ui,
                            "Reference:   ",
                            "Reference point to which the dimensions of the cuboid relate.",
                        );
                    }
                    GeomSelection::Cylinder => {
                        self.geom_selection_params.cylinder.specify(ui);
                        self.geom_selection_params.specify_reference(
                            ui,
                            "Reference:   ",
                            "Reference point to which the dimensions of the cylinder relate.",
                        );
                    }
                    GeomSelection::Sphere => {
                        self.geom_selection_params.sphere.specify(ui);
                        self.geom_selection_params.specify_reference(
                            ui,
                            "Center:   ",
                            "Center of the sphere.",
                        );
                    }
                }
            },
        );
    }

    /// Check that all parameters for geometric selection have been provided.
    pub(super) fn check_geometry_sanity(&self) -> bool {
        let shape_valid = match self.geom_selection {
            GeomSelection::None | GeomSelection::Cuboid => true,
            GeomSelection::Cylinder => self.geom_selection_params.cylinder.sanity_check(),
            GeomSelection::Sphere => self.geom_selection_params.sphere.sanity_check(),
        };

        let ref_selection_valid = match self.geom_selection {
            GeomSelection::None => true,
            _ => {
                self.geom_selection_params.reference_type != GeomReferenceType::Selection
                    || !self.geom_selection_params.ref_selection.is_empty()
            }
        };

        shape_valid && ref_selection_valid
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn convert_reference_type() {
        assert_eq!(
            GeomReferenceType::from(gorder::input::GeomReference::Point(Vector3D::new(
                5.0, 2.5, 3.5
            ))),
            GeomReferenceType::Point
        );

        assert_eq!(
            GeomReferenceType::from(gorder::input::GeomReference::Center),
            GeomReferenceType::Center
        );

        assert_eq!(
            GeomReferenceType::from(gorder::input::GeomReference::Selection(String::from(
                "@protein"
            ))),
            GeomReferenceType::Selection
        );
    }

    #[test]
    fn convert_geometric_selection() {
        assert_eq!(GeomSelection::from(None), GeomSelection::None);

        assert_eq!(
            GeomSelection::from(Some(
                gorder::input::Geometry::cuboid(
                    gorder::input::GeomReference::Point(Vector3D::new(5.0, 2.5, 3.5)),
                    [-3.0, 2.0],
                    [-2.0, 0.0],
                    [0.0, 4.0],
                )
                .unwrap()
            )),
            GeomSelection::Cuboid
        );

        assert_eq!(
            GeomSelection::from(Some(
                gorder::input::Geometry::sphere(gorder::input::GeomReference::Center, 5.0).unwrap()
            )),
            GeomSelection::Sphere
        );

        assert_eq!(
            GeomSelection::from(Some(
                gorder::input::Geometry::cylinder(
                    gorder::input::GeomReference::Selection(String::from("@protein")),
                    2.0,
                    [0.0, f32::INFINITY],
                    gorder::input::Axis::Z,
                )
                .unwrap()
            )),
            GeomSelection::Cylinder
        );
    }

    #[test]
    fn convert_geometric_params() {
        let params = GeomSelectionParams::from(Some(
            gorder::input::Geometry::cuboid(
                gorder::input::GeomReference::Point(Vector3D::new(5.0, 2.5, 3.5)),
                [-3.0, 2.0],
                [-2.5, f32::INFINITY],
                [0.0, 4.35],
            )
            .unwrap(),
        ));

        assert_eq!(params.reference_type, GeomReferenceType::Point);
        assert_relative_eq!(params.ref_point.x, 5.0);
        assert_relative_eq!(params.ref_point.y, 2.5);
        assert_relative_eq!(params.ref_point.z, 3.5);

        assert_relative_eq!(params.cuboid.minx, -3.0);
        assert_relative_eq!(params.cuboid.maxx, 2.0);
        assert_relative_eq!(params.cuboid.miny, -2.5);
        assert!(params.cuboid.maxy.is_infinite());
        assert_relative_eq!(params.cuboid.minz, 0.0);
        assert_relative_eq!(params.cuboid.maxz, 4.35);

        let params = GeomSelectionParams::from(Some(
            gorder::input::Geometry::sphere(gorder::input::GeomReference::Center, 5.0).unwrap(),
        ));

        assert_eq!(params.reference_type, GeomReferenceType::Center);
        assert_relative_eq!(params.sphere.radius, 5.0);

        let params = GeomSelectionParams::from(Some(
            gorder::input::Geometry::cylinder(
                gorder::input::GeomReference::Selection(String::from("@protein")),
                2.0,
                [-1.0, f32::INFINITY],
                gorder::input::Axis::Z,
            )
            .unwrap(),
        ));

        assert_eq!(params.reference_type, GeomReferenceType::Selection);
        assert_eq!(params.ref_selection, String::from("@protein"));

        assert_relative_eq!(params.cylinder.radius, 2.0);
        assert_relative_eq!(params.cylinder.start, -1.0);
        assert!(params.cylinder.end.is_infinite());
        assert_eq!(params.cylinder.orientation, Axis::Z);
    }
}
