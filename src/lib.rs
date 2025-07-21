use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

mod json;
mod jsondiff;
mod errors;
mod buffer;
use crate::json::{parse_json, JsonDiffErrorType, JsonValue};
use crate::jsondiff::diff_json_value;
use crate::errors::JsonDiffError;
use crate::buffer::ByteBuffer;

fn get_prop<'a>(
    obj: &'a JsonValue<'a>,
    key: &str,
) -> Option<&'a JsonValue<'a>> {
    match obj {
        JsonValue::Object(fields) => {
            fields.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
        }
        _ => None,
    }
}

pub fn diff_json_strs<B: ByteBuffer>(
    old_json: &str,
    new_json: &str,
    buf: &mut B,
) -> Result<(), JsonDiffError> {
    let old_val = parse_json(old_json)?;
    let new_val = parse_json(new_json)?;
    let mut tmp_buf = String::new();
    tmp_buf.push('{');
    let mut first = true;
    for prop in ["taxons", "characters", "states", "books"] {
        let old_p = get_prop(&old_val, prop).ok_or_else(|| JsonDiffError {
            error_type: JsonDiffErrorType::PropertyMissing,
        })?;
        let new_p = get_prop(&new_val, prop).ok_or_else(|| JsonDiffError {
            error_type: JsonDiffErrorType::PropertyMissing,
        })?;
        let mut prop_buf = String::new();
        diff_json_value(old_p, new_p, true, false, &mut prop_buf);
        if !prop_buf.is_empty() {
            if !first { tmp_buf.push(','); } else { first = false; }
            tmp_buf.push('"');
            tmp_buf.push_str(prop);
            tmp_buf.push_str("\":");
            tmp_buf.push_str(&prop_buf);
        }
    }
    tmp_buf.push('}');
    if !first {
        buf.push_str(&tmp_buf);
    }
    Ok(())
}

#[wasm_bindgen]
pub fn diff_hazo_json_strs(old_json: &str, new_json: &str, out: &Uint8Array) -> f64 {
    use crate::buffer::JsByteBuffer;
    let mut buf = JsByteBuffer::new(1024);
    let r = diff_json_strs(old_json, new_json, &mut buf);
    if let Err(e) = r {
        return -1.0 - (e.error_type as i8 as f64);
    }
    let result = buf.as_uint8array();
    out.set(&result, 0);
    buf.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_json_strs_no_diff() {
        let a = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let b = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let mut buf = String::new();
        diff_json_strs(a, b, &mut buf).unwrap();
        assert_eq!(buf, "{\"taxons\":{\"added\":[],\"removed\":[]},\"characters\":{\"added\":[],\"removed\":[]},\"states\":{\"added\":[],\"removed\":[]},\"books\":{\"added\":[],\"removed\":[]}}"
        );
    }

    #[test]
    fn test_diff_json_strs_added_taxon() {
        let a = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let b = r#"{"taxons":[{"id":1}],"characters":[],"states":[],"books":[]}"#;
        let mut buf = String::new();
        diff_json_strs(a, b, &mut buf).unwrap();
        assert!(buf.contains("\"taxons\":{\"added\":[{\"id\":1}],\"removed\":[]}"));
    }

    #[test]
    fn test_diff_json_strs_removed_book() {
        let a = r#"{"taxons":[],"characters":[],"states":[],"books":[{"id":42}]}"#;
        let b = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let mut buf = String::new();
        diff_json_strs(a, b, &mut buf).unwrap();
        assert!(buf.contains("\"books\":{\"added\":[],\"removed\":[{\"id\":42}]}"));
    }

    #[test]
    fn test_diff_json_strs_modified_state() {
        let a = r#"{"taxons":[],"characters":[],"states":[{"id":1,"name":"A"}],"books":[]}"#;
        let b = r#"{"taxons":[],"characters":[],"states":[{"id":1,"name":"B"}],"books":[]}"#;
        let mut buf = String::new();
        diff_json_strs(a, b, &mut buf).unwrap();
        assert!(buf.contains("\"states\":{\"added\":[],\"removed\":[],\"modified\":[{\"name\":{\"old\":\"A\",\"new\":\"B\"}}]}"));
    }

    #[test]
    fn test_diff_json_strs_missing_property() {
        let a = r#"{"taxons":[],"characters":[],"states":[]}"#;
        let b = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let mut buf = String::new();
        let err = diff_json_strs(a, b, &mut buf);
        assert!(err.is_err());
    }
}
