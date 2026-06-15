//! Generate convergence data CSVs for Phase 2 validation plots.
//!
//! Produces four CSV files in `docs/phase2-matrix-fill/figures/`:
//! - `v_thin_001.csv` — self-impedance vs wire radius (thin-wire convergence)
//! - `v_quad_001.csv` — self-impedance vs quadrature order
//! - `v_quad_002.csv` — near-neighbor mutual impedance vs quadrature order
//! - `v_quad_003.csv` — far off-diagonal mutual impedance vs quadrature order
//!
//! Usage:
//!     cargo run --example convergence_data -p arcanum-matrix-fill

use arcanum_geometry::mesh::{
    CurveParams, GroundDescriptor, GroundType, LinearParams, Material, Mesh, Segment, TagMap,
};
use arcanum_matrix_fill::{fill_impedance_matrix, MatrixFillConfig};
use nalgebra::Vector3;
use std::fs;
use std::io::Write;
use std::path::Path;

const FREQ: f64 = 300e6;
const SEG_LENGTH: f64 = 0.05;
const A_MOD: f64 = 0.005; // Wire radius for quad tests

fn straight_wire_mesh(n_segments: usize, seg_length: f64, wire_radius: f64) -> Mesh {
    let segments: Vec<Segment> = (0..n_segments)
        .map(|i| {
            let z0 = i as f64 * seg_length;
            let z1 = z0 + seg_length;
            Segment {
                curve: CurveParams::Linear(LinearParams {
                    start: Vector3::new(0.0, 0.0, z0),
                    end: Vector3::new(0.0, 0.0, z1),
                }),
                wire_radius,
                material: Material::PEC,
                tag: 1,
                segment_index: i,
                wire_index: 0,
                is_image: false,
            }
        })
        .collect();

    Mesh {
        segments,
        junctions: vec![],
        endpoint_junction: vec![],
        ground: GroundDescriptor {
            ground_type: GroundType::None,
            conductivity: None,
            permittivity: None,
            images_generated: false,
        },
        tag_map: TagMap::default(),
    }
}

/// V-THIN-001: Self-impedance vs wire radius.
///
/// Radii: a = Δ × {0.1, 0.01, 0.001, 0.0001}
/// Single segment, Δ=0.05m, f=300MHz.
fn generate_v_thin_001(out_dir: &Path) {
    println!("Generating V-THIN-001...");
    let config = MatrixFillConfig::default();

    let a_over_delta_values = [0.1, 0.01, 0.001, 0.0001];

    let mut file = fs::File::create(out_dir.join("v_thin_001.csv")).unwrap();
    writeln!(file, "a_over_delta,re_z,im_z,abs_z").unwrap();

    for &a_over_delta in &a_over_delta_values {
        let a = SEG_LENGTH * a_over_delta;
        let mesh = straight_wire_mesh(1, SEG_LENGTH, a);
        let z = fill_impedance_matrix(&mesh, FREQ, &config);
        let val = z.read(0, 0);
        writeln!(
            file,
            "{},{},{},{}",
            a_over_delta,
            val.re,
            val.im,
            val.abs()
        )
        .unwrap();
    }
    println!("  -> v_thin_001.csv");
}

/// V-QUAD-001: Self-impedance Z[0,0] vs quadrature order.
///
/// Single segment, Δ=0.05m, a=0.005m, orders = {4, 8, 16, 32, 64}.
fn generate_v_quad_001(out_dir: &Path) {
    println!("Generating V-QUAD-001...");
    let mesh = straight_wire_mesh(1, SEG_LENGTH, A_MOD);
    let orders = [4, 8, 16, 32, 64];

    let mut file = fs::File::create(out_dir.join("v_quad_001.csv")).unwrap();
    writeln!(file, "order,re_z,im_z,abs_z").unwrap();

    for &order in &orders {
        let config = MatrixFillConfig {
            quadrature_order_near_singular: order,
            ..MatrixFillConfig::default()
        };
        let z = fill_impedance_matrix(&mesh, FREQ, &config);
        let val = z.read(0, 0);
        writeln!(file, "{},{},{},{}", order, val.re, val.im, val.abs()).unwrap();
    }
    println!("  -> v_quad_001.csv");
}

/// V-QUAD-002: Near-neighbor mutual impedance Z[0,1] vs quadrature order.
///
/// Two adjacent segments, Δ=0.05m, a=0.005m, orders = {4, 8, 16, 32, 64}.
fn generate_v_quad_002(out_dir: &Path) {
    println!("Generating V-QUAD-002...");
    let mesh = straight_wire_mesh(2, SEG_LENGTH, A_MOD);
    let orders = [4, 8, 16, 32, 64];

    let mut file = fs::File::create(out_dir.join("v_quad_002.csv")).unwrap();
    writeln!(file, "order,re_z,im_z,abs_z").unwrap();

    for &order in &orders {
        let config = MatrixFillConfig {
            quadrature_order_near_singular: order,
            ..MatrixFillConfig::default()
        };
        let z = fill_impedance_matrix(&mesh, FREQ, &config);
        let val = z.read(0, 1);
        writeln!(file, "{},{},{},{}", order, val.re, val.im, val.abs()).unwrap();
    }
    println!("  -> v_quad_002.csv");
}

/// V-QUAD-003: Far off-diagonal mutual impedance Z[0,10] vs quadrature order.
///
/// 11-segment dipole, Δ=0.05m, a=0.005m, orders = {4, 8, 16, 32, 64}.
fn generate_v_quad_003(out_dir: &Path) {
    println!("Generating V-QUAD-003...");
    let mesh = straight_wire_mesh(11, SEG_LENGTH, A_MOD);
    let orders = [4, 8, 16, 32, 64];

    let mut file = fs::File::create(out_dir.join("v_quad_003.csv")).unwrap();
    writeln!(file, "order,re_z,im_z,abs_z").unwrap();

    for &order in &orders {
        let config = MatrixFillConfig {
            quadrature_order_regular: order,
            ..MatrixFillConfig::default()
        };
        let z = fill_impedance_matrix(&mesh, FREQ, &config);
        let val = z.read(0, 10);
        writeln!(file, "{},{},{},{}", order, val.re, val.im, val.abs()).unwrap();
    }
    println!("  -> v_quad_003.csv");
}

fn main() {
    let out_dir = Path::new("docs/phase2-matrix-fill/figures");
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    generate_v_thin_001(out_dir);
    generate_v_quad_001(out_dir);
    generate_v_quad_002(out_dir);
    generate_v_quad_003(out_dir);

    println!("\nAll convergence CSVs written to {}", out_dir.display());
}
