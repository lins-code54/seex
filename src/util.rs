use encoding_rs::GBK;
use serde_json::Value;

/// Convert EasyEDA's 3D model rotation into the angle written to KiCad footprints and Altium
/// 3D bodies.
///
/// EasyEDA stores the model angle in its footprint frame, where `+Y` runs down the screen, while
/// both targets rotate the model around the Z axis of a frame whose `+Y` points up. The same
/// visual rotation therefore has the opposite sign. Verified with KiCad renders: a footprint
/// whose model is stored with 90 degrees only lands on its pads with a 270 degree rotation, and
/// a stored 0 stays 0.
///
/// The translation is deliberately left at zero: EasyEDA already stores the model where it
/// belongs, and the offsets carried in `model_3d.transform` / `c_origin` move footprints that
/// render correctly today (C144198, C668205, C725790) off the pads by about a millimetre.
///
/// Only the EasyEDA angle feeds this conversion. Real Altium libraries store the same number in
/// both `MODEL.3D.ROTZ` and the model table entry, which is what the exporter mirrors; the
/// direction cannot be checked without Altium itself, and for every package whose pad array is
/// symmetric about the X axis (i.e. all chip packages) a 180 degree error is invisible in the
/// 3D view.
pub fn model_rotation_z(easyeda_degrees: f64) -> f64 {
    if !easyeda_degrees.is_finite() {
        return 0.0;
    }
    let converted = (-easyeda_degrees).rem_euclid(360.0);
    // `rem_euclid` can hand back -0.0 for a zero angle; normalise it so exported files stay stable.
    if converted == 0.0 { 0.0 } else { converted }
}

pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect();

    let trimmed = cleaned.trim_matches(|ch| ch == ' ' || ch == '.').trim();
    if trimmed.is_empty() {
        "component".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

pub fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(repair_misdecoded_gbk_text(text)),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

pub fn nested_string(value: &Value, path: &[&str]) -> Option<String> {
    nested_value(value, path).and_then(value_to_string)
}

pub fn repair_misdecoded_gbk_text(text: &str) -> String {
    if text.is_empty() || text.chars().any(is_cjk) {
        return text.to_string();
    }

    let mut bytes = Vec::with_capacity(text.len());
    let mut suspicious_count = 0usize;
    for character in text.chars() {
        let Some(byte) = windows_1252_byte(character) else {
            return text.to_string();
        };
        if byte >= 0x80 {
            suspicious_count += 1;
        }
        bytes.push(byte);
    }

    if suspicious_count < 2 {
        return text.to_string();
    }

    let (decoded, _, had_errors) = GBK.decode(&bytes);
    if had_errors || !decoded.chars().any(is_cjk) {
        return text.to_string();
    }

    decoded.into_owned()
}

fn windows_1252_byte(character: char) -> Option<u8> {
    match character {
        '\u{20AC}' => Some(0x80),
        '\u{201A}' => Some(0x82),
        '\u{0192}' => Some(0x83),
        '\u{201E}' => Some(0x84),
        '\u{2026}' => Some(0x85),
        '\u{2020}' => Some(0x86),
        '\u{2021}' => Some(0x87),
        '\u{02C6}' => Some(0x88),
        '\u{2030}' => Some(0x89),
        '\u{0160}' => Some(0x8A),
        '\u{2039}' => Some(0x8B),
        '\u{0152}' => Some(0x8C),
        '\u{017D}' => Some(0x8E),
        '\u{2018}' => Some(0x91),
        '\u{2019}' => Some(0x92),
        '\u{201C}' => Some(0x93),
        '\u{201D}' => Some(0x94),
        '\u{2022}' => Some(0x95),
        '\u{2013}' => Some(0x96),
        '\u{2014}' => Some(0x97),
        '\u{02DC}' => Some(0x98),
        '\u{2122}' => Some(0x99),
        '\u{0161}' => Some(0x9A),
        '\u{203A}' => Some(0x9B),
        '\u{0153}' => Some(0x9C),
        '\u{017E}' => Some(0x9E),
        '\u{0178}' => Some(0x9F),
        ch if (ch as u32) <= 0xFF => Some(ch as u8),
        _ => None,
    }
}

fn is_cjk(character: char) -> bool {
    matches!(
        character as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
    )
}

pub fn split_obj_and_mtl(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();
    let mut mtl_lines = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("newmtl") {
            mtl_lines.push(line.to_string());
            let mut j = i + 1;
            while j < lines.len() {
                let next_line = lines[j];
                let token = next_line.split_whitespace().next().unwrap_or_default();
                if matches!(
                    token,
                    "newmtl" | "v" | "vt" | "vn" | "f" | "o" | "g" | "s" | "usemtl" | "mtllib"
                ) {
                    break;
                }
                mtl_lines.push(next_line.to_string());
                j += 1;
            }
        }
        i += 1;
    }

    let mut obj_text = lines.join("\n");
    if !obj_text.ends_with('\n') {
        obj_text.push('\n');
    }

    let mut mtl_text = mtl_lines.join("\n");
    if !mtl_text.is_empty() && !mtl_text.ends_with('\n') {
        mtl_text.push('\n');
    }

    (obj_text, mtl_text)
}

#[cfg(test)]
mod tests {
    use super::{
        model_rotation_z, repair_misdecoded_gbk_text, sanitize_filename, split_obj_and_mtl,
    };

    #[test]
    fn converts_easyeda_model_rotation_into_body_rotation() {
        assert_eq!(model_rotation_z(0.0), 0.0);
        assert_eq!(model_rotation_z(90.0), 270.0);
        assert_eq!(model_rotation_z(180.0), 180.0);
        assert_eq!(model_rotation_z(270.0), 90.0);
        assert_eq!(model_rotation_z(-90.0), 90.0);
        assert_eq!(model_rotation_z(360.0), 0.0);
        assert_eq!(model_rotation_z(f64::NAN), 0.0);
    }

    #[test]
    fn sanitizes_windows_unsafe_characters() {
        assert_eq!(sanitize_filename("A<B>:C*.step"), "A_B__C_.step");
        assert_eq!(sanitize_filename(" .. "), "component");
    }

    #[test]
    fn splits_embedded_mtl_sections() {
        let input = "newmtl body\nKd 0.8 0.8 0.8\nv 0 0 0\nf 1 1 1\n";
        let (obj_text, mtl_text) = split_obj_and_mtl(input);
        assert!(obj_text.contains("v 0 0 0"));
        assert!(mtl_text.contains("newmtl body"));
        assert!(mtl_text.contains("Kd 0.8 0.8 0.8"));
    }

    #[test]
    fn repairs_gbk_text_decoded_as_windows_1252() {
        assert_eq!(
            repair_misdecoded_gbk_text("\u{00b5}\u{00e7}\u{00d7}\u{00e8}"),
            "\u{7535}\u{963b}"
        );
        assert_eq!(repair_misdecoded_gbk_text("10uF"), "10uF");
    }
}
