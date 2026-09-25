//! Board-plane placement of the STEP models that ship with an EasyEDA footprint.
//!
//! LCEDA authors every STEP model in its own local frame, and that frame does not always put the
//! part's underside on the board plane: `C2904739` (SOP-16) and `C9864` (ESOP-8) are centred on Z
//! and sink ~0.8 mm into the board, `C37593` (MSOP-10) leaves its lead feet 0.55 mm below the
//! origin, while most parts (for example `C128601`, `C190794`, `C668205`) already rest on Z = 0.
//! Both exporters embed the downloaded STEP unchanged, so the parts below the plane render buried
//! in the board while everything else looks right.
//!
//! Neither target format can be trusted to fix that on its own: KiCad and Altium each have a model
//! Z-offset field, but they disagree about its sign and the offset only applies when the consumer
//! keeps the STEP as a separate model reference. A STEP file itself has no origin to move, so the
//! correction is applied to the geometry: the underside of the part is measured and every vertex is
//! shifted up until it rests on Z = 0. Embedded models, exported `.step` files and both CAD tools
//! then agree without depending on any tool's own convention.
//!
//! Call [`place_step_on_board`] before writing or embedding a downloaded STEP model.

use std::collections::BTreeMap;
use std::ops::Range;

/// Height of one histogram bucket when locating the underside of a part.
const UNDERSIDE_BIN_MM: f64 = 0.01;
/// Smaller corrections are measurement noise rather than a misplaced model.
const MIN_LIFT_MM: f64 = 0.05;
/// Larger corrections come from stray geometry (a hidden solid far below the part), not placement.
const MAX_LIFT_MM: f64 = 2.0;
/// Below this vertex count there is not enough geometry to measure anything.
const MIN_VERTICES: usize = 16;
/// A bucket counts as the underside only when a few vertices share it.
const MIN_CLUSTER_VERTICES: usize = 3;
const MIN_CLUSTER_FRACTION: f64 = 0.001;

const CARTESIAN_POINT: &[u8] = b"CARTESIAN_POINT";

#[derive(Debug, Clone)]
struct ModelPoint {
    z: f64,
    /// Span of the Z literal in the STEP source, used to rewrite it.
    z_span: Range<usize>,
}

/// Shift a downloaded STEP model so its underside rests on the board plane.
///
/// Returns the input unchanged when the model already rests on Z = 0, when it carries too little
/// geometry to measure, or when the measured correction is implausible (see the constants above).
pub fn place_step_on_board(step_data: &[u8]) -> Vec<u8> {
    let points = parse_model_points(step_data);
    let lift = underside_lift_mm(&points);
    if lift == 0.0 {
        return step_data.to_vec();
    }

    let mut shifted = Vec::with_capacity(step_data.len());
    let mut cursor = 0usize;
    for point in &points {
        shifted.extend_from_slice(&step_data[cursor..point.z_span.start]);
        shifted.extend_from_slice(format_step_real(point.z + lift).as_bytes());
        cursor = point.z_span.end;
    }
    shifted.extend_from_slice(&step_data[cursor..]);
    shifted
}

/// How far a model has to move up to rest on the board plane, in millimetres.
fn underside_lift_mm(points: &[ModelPoint]) -> f64 {
    if points.len() < MIN_VERTICES {
        return 0.0;
    }

    let minimum_cluster =
        ((points.len() as f64 * MIN_CLUSTER_FRACTION).ceil() as usize).max(MIN_CLUSTER_VERTICES);
    let mut buckets = BTreeMap::<i64, usize>::new();
    for point in points {
        *buckets.entry(z_bucket(point.z)).or_default() += 1;
    }

    // Buckets iterate in ascending order, so the first that holds a cluster is the lowest one.
    let Some((&bucket, _)) = buckets.iter().find(|(_, count)| **count >= minimum_cluster) else {
        return 0.0;
    };

    let lift = round_micrometres(-(bucket as f64 * UNDERSIDE_BIN_MM));
    if (MIN_LIFT_MM..=MAX_LIFT_MM).contains(&lift) {
        lift
    } else {
        0.0
    }
}

fn z_bucket(z: f64) -> i64 {
    (z / UNDERSIDE_BIN_MM).round() as i64
}

fn round_micrometres(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}

/// Every `CARTESIAN_POINT` with three coordinates, in file order.
fn parse_model_points(step_data: &[u8]) -> Vec<ModelPoint> {
    let mut points = Vec::new();
    let mut search_from = 0usize;

    while let Some(found) = find_from(step_data, CARTESIAN_POINT, search_from) {
        search_from = found + CARTESIAN_POINT.len();
        let Some(coordinates) = cartesian_point_coordinates(step_data, search_from) else {
            continue;
        };
        let mut values = coordinate_values(step_data, coordinates);
        if values.len() != 3 {
            // Parameter-space points carry two coordinates and are not part of the 3D body.
            continue;
        }
        let (z, z_span) = values.pop().expect("three coordinates");
        points.push(ModelPoint { z, z_span });
    }

    points
}

/// Span of the `( x, y, z )` group that follows a `CARTESIAN_POINT` label.
fn cartesian_point_coordinates(step_data: &[u8], from: usize) -> Option<Range<usize>> {
    // The record reads `CARTESIAN_POINT ( 'NONE', ( x, y, z ) )`: the first parenthesis opens the
    // parameter list, the second one the coordinates.
    let mut cursor = from;
    for _ in 0..2 {
        let open = step_data[cursor..].iter().position(|byte| *byte == b'(')?;
        cursor += open + 1;
    }
    let close = step_data[cursor..].iter().position(|byte| *byte == b')')?;
    Some(cursor..cursor + close)
}

/// Values and absolute spans of the comma separated coordinates in a point group.
fn coordinate_values(step_data: &[u8], span: Range<usize>) -> Vec<(f64, Range<usize>)> {
    let mut values = Vec::new();
    let mut start = span.start;

    for index in span.clone() {
        if step_data[index] == b',' {
            push_coordinate(step_data, start..index, &mut values);
            start = index + 1;
        }
    }
    push_coordinate(step_data, start..span.end, &mut values);

    values
}

fn push_coordinate(step_data: &[u8], span: Range<usize>, values: &mut Vec<(f64, Range<usize>)>) {
    let text = &step_data[span.clone()];
    let leading = text
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count();
    let trailing = text
        .iter()
        .rev()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count();
    if leading + trailing >= text.len() {
        return;
    }

    let trimmed = span.start + leading..span.end - trailing;
    let Some(value) = std::str::from_utf8(&step_data[trimmed.clone()])
        .ok()
        .and_then(|text| text.parse::<f64>().ok())
    else {
        return;
    };

    values.push((value, trimmed));
}

/// STEP requires a decimal point in every real; a fixed nine decimals is both valid and plenty.
fn format_step_real(value: f64) -> String {
    format!("{value:.9}")
}

fn find_from(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if from >= haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| from + offset)
}

#[cfg(test)]
mod tests {
    use super::{ModelPoint, parse_model_points, place_step_on_board, underside_lift_mm};

    fn step_with(heights: &[f64]) -> Vec<u8> {
        let mut text = String::from("ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\n");
        for (index, height) in heights.iter().enumerate() {
            text.push_str(&format!(
                "#{index} = CARTESIAN_POINT ( 'NONE',  ( 0.0, 0.0, {height} ) ) ;\n"
            ));
        }
        text.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
        text.into_bytes()
    }

    fn heights_of(step_data: &[u8]) -> Vec<f64> {
        parse_model_points(step_data)
            .into_iter()
            .map(|point| point.z)
            .collect()
    }

    fn lift_of(heights: &[f64]) -> f64 {
        let step_data = step_with(heights);
        underside_lift_mm(&parse_model_points(&step_data))
    }

    #[test]
    fn leaves_a_model_that_already_rests_on_the_board_untouched() {
        let heights: Vec<f64> = (0..100).map(|index| index as f64 * 0.01).collect();
        let step_data = step_with(&heights);

        assert_eq!(lift_of(&heights), 0.0);
        assert_eq!(place_step_on_board(&step_data), step_data);
    }

    #[test]
    fn lifts_a_model_whose_underside_sits_below_the_board() {
        // A flat underside 0.8 mm below the origin, like the centred LCEDA models.
        let mut heights = vec![-0.8; 60];
        heights.extend((0..140).map(|index| index as f64 * 0.01));
        let step_data = step_with(&heights);

        let shifted = place_step_on_board(&step_data);
        let shifted_heights = heights_of(&shifted);

        assert!((lift_of(&heights) - 0.8).abs() < 1e-6);
        assert!(shifted_heights[0] >= -0.001, "underside still sunk");
        assert!(
            (shifted_heights[shifted_heights.len() - 1] - (heights[heights.len() - 1] + 0.8)).abs()
                < 0.001
        );
        assert_eq!(shifted_heights.len(), heights.len());
    }

    #[test]
    fn measures_the_underside_from_the_lowest_cluster_of_vertices() {
        // A lone pair of vertices 0.07 mm below the real underside must not drive the lift.
        let mut heights = vec![-0.932, -0.932];
        heights.extend((0..144).map(|_| -0.86));
        heights.extend((0..500).map(|index| index as f64 * 0.002));

        assert!((lift_of(&heights) - 0.86).abs() < 1e-6);
    }

    #[test]
    fn ignores_a_stray_solid_far_below_the_part() {
        let mut heights: Vec<f64> = (0..400).map(|index| index as f64 * 0.005).collect();
        heights.extend((0..22).map(|_| -12.924));

        assert_eq!(lift_of(&heights), 0.0);
    }

    #[test]
    fn ignores_corrections_below_the_measurement_threshold() {
        let mut heights = vec![-0.01; 40];
        heights.extend((0..60).map(|index| 0.5 + index as f64 * 0.01));

        assert_eq!(lift_of(&heights), 0.0);
    }

    #[test]
    fn ignores_models_without_enough_geometry() {
        assert_eq!(lift_of(&[0.0, -0.5, -0.5, -0.5]), 0.0);

        let step_data = b"ISO-10303-21;\nENDSEC;\n".to_vec();
        assert_eq!(place_step_on_board(&step_data), step_data);
    }

    #[test]
    fn only_rewrites_three_dimensional_points() {
        let text = concat!(
            "DATA;\n",
            "#1 = CARTESIAN_POINT ( 'NONE',  ( 0.0, 0.0, -0.8000000000000000 ) ) ;\n",
            "#2 = CARTESIAN_POINT ( 'NONE',  ( 0.5, 0.5 ) ) ;\n"
        );
        let heights: Vec<f64> = (0..20).map(|_| -0.8).collect();
        let mut step_data = text.as_bytes().to_vec();
        step_data.extend(step_with(&heights));
        step_data.extend_from_slice(b"ENDSEC;\n");

        let shifted = place_step_on_board(&step_data);
        let shifted_text = String::from_utf8(shifted).expect("utf8");

        assert!(shifted_text.contains("#2 = CARTESIAN_POINT ( 'NONE',  ( 0.5, 0.5 ) ) ;"));
        assert!(!shifted_text.contains("-0.800000"));
    }

    #[test]
    fn keeps_the_vertex_count_and_the_entity_layout() {
        let heights: Vec<f64> = (0..250).map(|index| -0.62 + index as f64 * 0.004).collect();
        let step_data = step_with(&heights);

        let shifted = place_step_on_board(&step_data);

        assert_eq!(
            shifted
                .windows(b"CARTESIAN_POINT".len())
                .filter(|window| *window == b"CARTESIAN_POINT")
                .count(),
            heights.len()
        );
        assert!(shifted.ends_with(b"ENDSEC;\nEND-ISO-10303-21;\n"));
    }

    #[test]
    fn parses_points_with_tight_spacing_and_exponents() {
        let points = parse_model_points(
            b"#7=CARTESIAN_POINT('NONE',(1.5E-01,-2,3.25E+00));#8=CARTESIAN_POINT('NONE',(0,0,-8.6E-01));",
        );

        assert_eq!(points.len(), 2);
        assert_eq!(points[0].z, 3.25);
        assert_eq!(points[1].z, -0.86);
    }

    #[test]
    fn rewrites_only_the_z_coordinate() {
        let step_data = b"#1 = CARTESIAN_POINT ( 'NONE',  ( 1.25, -3.5, -0.9 ) ) ;".to_vec();
        let points: Vec<ModelPoint> = parse_model_points(&step_data);

        assert_eq!(points.len(), 1);
        assert_eq!(&step_data[points[0].z_span.clone()], b"-0.9");
    }
}
