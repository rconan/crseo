//! Generate circular mesh using Delaunay triangulation

use triangle_rs::{Builder, Delaunay};

/// Circular mesh
///
/// The mesh is builder based on the diameter odf the enclosing circles
///  and a maximum triangle area constraint
pub struct Mesh {
    diameter: f64,
    max_area_triangle: f64,
}
impl Mesh {
    pub fn new(diameter: f64, max_area_triangle: f64) -> Builder {
        let perimeter = std::f64::consts::PI * diameter;
        let side = (4f64 * max_area_triangle / 3f64.sqrt()).sqrt();
        let n = (perimeter / side).round() as usize;
        let r = diameter * 0.5;
        let vertices: Vec<f64> = (0..n)
            .map(|i| 2. * i as f64 * std::f64::consts::PI / n as f64)
            .map(|o| o.sin_cos())
            .flat_map(|(y, x)| vec![r * x, r * y])
            .collect();
        Builder::new()
            .add_nodes(vertices.as_slice())
            .set_switches(&format!("Qqa{:}", max_area_triangle))
    }
}
