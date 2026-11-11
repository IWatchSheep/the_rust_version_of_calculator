//! Geometry operators ported from `operators_one_operator_per_file/geometry/geometry/*`.
//! One function per operator, named after the folder.  `operator_keys/make_rust_ops.py` wires each function
//! to its catalog entry (key, name, inputs, defaults) in `generated.rs`.

use crate::value::Value;
use std::f64::consts::PI;

type R = Result<Value, String>;

fn n(x: f64) -> R {
    Ok(Value::Number(x))
}

pub fn arc_length(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn cone_surface_area(a: &[f64]) -> R {
    n(PI * a[0] * (a[0] + a[1]))
}
pub fn cone_volume(a: &[f64]) -> R {
    n(PI * a[0].powi(2) * a[1] / 3.0)
}
pub fn cube_surface_area(a: &[f64]) -> R {
    n(6.0 * a[0].powi(2))
}
pub fn cube_volume(a: &[f64]) -> R {
    n(a[0].powi(3))
}
pub fn cylinder_surface_area(a: &[f64]) -> R {
    n(2.0 * PI * a[0] * (a[0] + a[1]))
}
pub fn degrees_to_radians_geometry(a: &[f64]) -> R {
    n(a[0].to_radians())
}
pub fn heron_s_formula(a: &[f64]) -> R {
    let s = (a[0] + a[1] + a[2]) / 2.0;
    n((s * (s - a[0]) * (s - a[1]) * (s - a[2])).sqrt())
}
pub fn heron_semiperimeter(a: &[f64]) -> R {
    n((a[0] + a[1] + a[2]) / 2.0)
}
pub fn heron_area(a: &[f64]) -> R {
    // s, a, b, c
    n((a[0] * (a[0] - a[1]) * (a[0] - a[2]) * (a[0] - a[3])).sqrt())
}
pub fn law_of_cosines(a: &[f64]) -> R {
    n((a[0] * a[0] + a[1] * a[1] - 2.0 * a[0] * a[1] * a[2].cos()).sqrt())
}
pub fn law_of_sines(a: &[f64]) -> R {
    n(a[0] * a[2].sin() / a[1].sin())
}
pub fn midpoint_formula(a: &[f64]) -> R {
    Ok(Value::Pair((a[0] + a[2]) / 2.0, (a[1] + a[3]) / 2.0))
}
pub fn parallelogram_area(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn polygon_interior_angle_sum(a: &[f64]) -> R {
    n((a[0] - 2.0) * 180.0)
}
pub fn pyramid_volume(a: &[f64]) -> R {
    n(a[0] * a[1] / 3.0)
}
pub fn radians_to_degrees_geometry(a: &[f64]) -> R {
    n(a[0].to_degrees())
}
pub fn rectangle_perimeter(a: &[f64]) -> R {
    n(2.0 * (a[0] + a[1]))
}
pub fn rectangular_prism_surface_area(a: &[f64]) -> R {
    n(2.0 * (a[0] * a[1] + a[0] * a[2] + a[1] * a[2]))
}
pub fn rectangular_prism_volume(a: &[f64]) -> R {
    n(a[0] * a[1] * a[2])
}
pub fn regular_polygon_area(a: &[f64]) -> R {
    n(0.5 * a[0] * a[1])
}
pub fn rhombus_area(a: &[f64]) -> R {
    n(0.5 * a[0] * a[1])
}
pub fn sector_area(a: &[f64]) -> R {
    n(0.5 * a[0].powi(2) * a[1])
}
pub fn slope(a: &[f64]) -> R {
    n((a[3] - a[1]) / (a[2] - a[0]))
}
pub fn sphere_surface_area(a: &[f64]) -> R {
    n(4.0 * PI * a[0].powi(2))
}
pub fn square_area(a: &[f64]) -> R {
    n(a[0].powi(2))
}
pub fn trapezoid_area(a: &[f64]) -> R {
    n(0.5 * (a[0] + a[1]) * a[2])
}
