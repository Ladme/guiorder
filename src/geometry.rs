// Released under MIT License.
// Copyright (c) 2025 Ladislav Bartos

//! Parameters for geometric selection.

use std::fmt::Display;

use eframe::egui::{self, ComboBox, DragValue, Response, RichText, Ui};
use gorder::{input::Axis, prelude::Vector3D};

use crate::{error::ConversionError, GuiAnalysis};

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

impl TryFrom<&GuiAnalysis> for Option<gorder::input::Geometry> {
    type Error = ConversionError;

    fn try_from(value: &GuiAnalysis) -> Result<Self, Self::Error> {
        let params = &value.geom_selection_params;
        let reference = gorder::input::GeomReference::from(params);

        match value.geom_selection {
            GeomSelection::None => Ok(None),
            GeomSelection::Cuboid => Ok(Some(
                gorder::input::Geometry::cuboid(
                    reference,
                    [params.cuboid.minx, params.cuboid.maxx],
                    [params.cuboid.miny, params.cuboid.maxy],
                    [params.cuboid.minz, params.cuboid.maxz],
                )
                .map_err(|e| ConversionError::InvalidGeometryParams(e.to_string()))?,
            )),
            GeomSelection::Cylinder => Ok(Some(
                gorder::input::Geometry::cylinder(
                    reference,
                    params.cylinder.radius,
                    [params.cylinder.start, params.cylinder.end],
                    params.cylinder.orientation,
                )
                .map_err(|e| ConversionError::InvalidGeometryParams(e.to_string()))?,
            )),
            GeomSelection::Sphere => Ok(Some(
                gorder::input::Geometry::sphere(reference, params.sphere.radius)
                    .map_err(|e| ConversionError::InvalidGeometryParams(e.to_string()))?,
            )),
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

impl From<&GeomSelectionParams> for gorder::input::GeomReference {
    fn from(value: &GeomSelectionParams) -> Self {
        match value.reference_type {
            GeomReferenceType::Point => {
                gorder::input::GeomReference::Point(value.ref_point.clone())
            }
            GeomReferenceType::Selection => {
                gorder::input::GeomReference::Selection(value.ref_selection.clone())
            }
            GeomReferenceType::Center => gorder::input::GeomReference::Center,
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
        if value.is_infinite() && response.dragged() {
            *value = target;
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
                                    variant,
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
    fn gorder_to_guiorder_ref_point() {
        assert_eq!(
            GeomReferenceType::from(gorder::input::GeomReference::Point(Vector3D::new(
                5.0, 2.5, 3.5
            ))),
            GeomReferenceType::Point
        );
    }

    #[test]
    fn gorder_to_guiorder_ref_center() {
        assert_eq!(
            GeomReferenceType::from(gorder::input::GeomReference::Center),
            GeomReferenceType::Center
        );
    }

    #[test]
    fn gorder_to_guiorder_ref_selection() {
        assert_eq!(
            GeomReferenceType::from(gorder::input::GeomReference::Selection(String::from(
                "@protein"
            ))),
            GeomReferenceType::Selection
        );
    }

    #[test]
    fn gorder_to_guiorder_geometry_type_none() {
        assert_eq!(GeomSelection::from(None), GeomSelection::None);
    }

    #[test]
    fn gorder_to_guiorder_geometry_type_cuboid() {
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
    }

    #[test]
    fn gorder_to_guiorder_geometry_type_cylinder() {
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
    fn gorder_to_guiorder_geometry_type_sphere() {
        assert_eq!(
            GeomSelection::from(Some(
                gorder::input::Geometry::sphere(gorder::input::GeomReference::Center, 5.0).unwrap()
            )),
            GeomSelection::Sphere
        );
    }

    #[test]
    fn gorder_to_guiorder_geometry_params_cuboid() {
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
    }

    #[test]
    fn gorder_to_guiorder_geometry_params_cylinder() {
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

    #[test]
    fn gorder_to_guiorder_geometry_params_sphere() {
        let params = GeomSelectionParams::from(Some(
            gorder::input::Geometry::sphere(gorder::input::GeomReference::Center, 5.0).unwrap(),
        ));

        assert_eq!(params.reference_type, GeomReferenceType::Center);
        assert_relative_eq!(params.sphere.radius, 5.0);
    }

    #[test]
    fn guiorder_to_gorder_ref_point() {
        let params = GeomSelectionParams {
            reference_type: GeomReferenceType::Point,
            ref_point: Vector3D::new(5.0, -3.2, 1.8),
            ..Default::default()
        };

        let converted = gorder::input::GeomReference::from(&params);
        match converted {
            gorder::input::GeomReference::Point(point) => {
                assert_relative_eq!(point.x, 5.0);
                assert_relative_eq!(point.y, -3.2);
                assert_relative_eq!(point.z, 1.8);
            }
            _ => panic!("Invalid GeomReference returned."),
        }
    }

    #[test]
    fn guiorder_to_gorder_ref_center() {
        let params = GeomSelectionParams {
            reference_type: GeomReferenceType::Center,
            ..Default::default()
        };

        let converted = gorder::input::GeomReference::from(&params);
        assert!(matches!(converted, gorder::input::GeomReference::Center));
    }

    #[test]
    fn guiorder_to_gorder_ref_selection() {
        let params = GeomSelectionParams {
            reference_type: GeomReferenceType::Selection,
            ref_selection: String::from("@protein"),
            ..Default::default()
        };

        let converted = gorder::input::GeomReference::from(&params);
        match converted {
            gorder::input::GeomReference::Selection(query) => {
                assert_eq!(query, String::from("@protein"));
            }
            _ => panic!("Invalid GeomReference returned."),
        }
    }

    #[test]
    fn guiorder_to_gorder_no_geometry() {
        let params = GuiAnalysis {
            geom_selection: GeomSelection::None,
            ..Default::default()
        };

        let converted = Option::<gorder::input::Geometry>::try_from(&params).unwrap();
        assert!(converted.is_none());
    }

    #[test]
    fn guiorder_to_gorder_cuboid_geometry() {
        let params = GuiAnalysis {
            geom_selection: GeomSelection::Cuboid,
            geom_selection_params: GeomSelectionParams {
                cuboid: CuboidParams {
                    minx: f32::NEG_INFINITY,
                    maxx: 6.3,
                    miny: -3.5,
                    maxy: -1.4,
                    minz: 0.0,
                    maxz: f32::INFINITY,
                },
                ref_point: Vector3D::new(5.0, -3.2, 1.8),
                reference_type: GeomReferenceType::Point,
                ..Default::default()
            },
            ..Default::default()
        };

        let converted_geometry = Option::<gorder::input::Geometry>::try_from(&params)
            .unwrap()
            .unwrap();

        match converted_geometry {
            gorder::input::Geometry::Cuboid(converted_params) => {
                let reference = converted_params.reference();
                match reference {
                    gorder::input::GeomReference::Point(p) => {
                        assert_relative_eq!(p.x, 5.0);
                        assert_relative_eq!(p.y, -3.2);
                        assert_relative_eq!(p.z, 1.8);
                    }
                    _ => panic!("Invalid GeomReference returned."),
                }

                assert!(converted_params.xdim()[0].is_infinite());
                assert_relative_eq!(converted_params.xdim()[1], 6.3);
                assert_relative_eq!(converted_params.ydim()[0], -3.5);
                assert_relative_eq!(converted_params.ydim()[1], -1.4);
                assert_relative_eq!(converted_params.zdim()[0], 0.0);
                assert!(converted_params.zdim()[1].is_infinite());
            }
            _ => panic!("Invalid geometry."),
        }
    }

    #[test]
    fn guiorder_to_gorder_cylinder_geometry() {
        let params = GuiAnalysis {
            geom_selection: GeomSelection::Cylinder,
            geom_selection_params: GeomSelectionParams {
                cylinder: CylinderParams {
                    radius: 2.56,
                    start: f32::NEG_INFINITY,
                    end: 1.7,
                    orientation: Axis::X,
                },
                ref_selection: String::from("@protein"),
                reference_type: GeomReferenceType::Selection,
                ..Default::default()
            },
            ..Default::default()
        };

        let converted_geometry = Option::<gorder::input::Geometry>::try_from(&params)
            .unwrap()
            .unwrap();

        match converted_geometry {
            gorder::input::Geometry::Cylinder(converted_params) => {
                let reference = converted_params.reference();
                match reference {
                    gorder::input::GeomReference::Selection(query) => {
                        assert_eq!(query, &String::from("@protein"));
                    }
                    _ => panic!("Invalid GeomReference returned."),
                }

                assert_relative_eq!(converted_params.radius(), 2.56);
                assert!(converted_params.span()[0].is_infinite());
                assert_relative_eq!(converted_params.span()[1], 1.7);
                assert_eq!(converted_params.orientation(), Axis::X);
            }
            _ => panic!("Invalid geometry."),
        }
    }

    #[test]
    fn guiorder_to_gorder_sphere_geometry() {
        let params = GuiAnalysis {
            geom_selection: GeomSelection::Sphere,
            geom_selection_params: GeomSelectionParams {
                sphere: SphereParams { radius: 1.43 },
                reference_type: GeomReferenceType::Center,
                ..Default::default()
            },
            ..Default::default()
        };

        let converted_geometry = Option::<gorder::input::Geometry>::try_from(&params)
            .unwrap()
            .unwrap();

        match converted_geometry {
            gorder::input::Geometry::Sphere(converted_params) => {
                let reference = converted_params.reference();
                assert!(matches!(reference, gorder::input::GeomReference::Center));
                assert_relative_eq!(converted_params.radius(), 1.43);
            }
            _ => panic!("Invalid geometry."),
        }
    }
}
