use wasm_bindgen::prelude::*;

mod json;
mod jsondiff;
mod errors;
use crate::json::{parse_json, JsonValue};
use crate::jsondiff::diff_json_value;
use crate::errors::JsonDiffError;

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

pub fn diff_json_strs(
    old_json: &str,
    new_json: &str,
) -> Result<String, JsonDiffError> {
    let old_val = parse_json(old_json)?;
    let new_val = parse_json(new_json)?;
    let mut result_fields = Vec::new();
    for prop in ["taxons", "characters", "states", "books"] {
        let old_p = get_prop(&old_val, prop).ok_or_else(|| JsonDiffError {
            error_type: json::JsonDiffErrorType::PropertyMissing,
        })?;
        let new_p = get_prop(&new_val, prop).ok_or_else(|| JsonDiffError {
            error_type: json::JsonDiffErrorType::PropertyMissing,
        })?;
        if let Some(diff) = diff_json_value(old_p, new_p, true, false) {
            result_fields.push(format!("\"{}\":{}", prop, diff));
        }
    }
    if result_fields.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("{{{}}}", result_fields.join(",")))
    }
}

#[wasm_bindgen]
pub fn diff_hazo_json_strs(old_json: &str, new_json: &str) -> Result<String, String> {
    diff_json_strs(old_json, new_json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_json_strs_no_diff() {
        let a = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let b = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let diff = diff_json_strs(a, b).unwrap();
        assert_eq!(diff, "{\"taxons\":{\"added\":[],\"removed\":[]},\"characters\":{\"added\":[],\"removed\":[]},\"states\":{\"added\":[],\"removed\":[]},\"books\":{\"added\":[],\"removed\":[]}}"
        );
    }

    #[test]
    fn test_diff_json_strs_added_taxon() {
        let a = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let b = r#"{"taxons":[{"id":1}],"characters":[],"states":[],"books":[]}"#;
        let diff = diff_json_strs(a, b).unwrap();
        assert!(diff.contains("\"taxons\":{\"added\":[{\"id\":1}],\"removed\":[]}"));
    }

    #[test]
    fn test_diff_json_strs_removed_book() {
        let a = r#"{"taxons":[],"characters":[],"states":[],"books":[{"id":42}]}"#;
        let b = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let diff = diff_json_strs(a, b).unwrap();
        assert!(diff.contains("\"books\":{\"added\":[],\"removed\":[{\"id\":42}]}"));
    }

    #[test]
    fn test_diff_json_strs_modified_state() {
        let a = r#"{"taxons":[],"characters":[],"states":[{"id":1,"name":"A"}],"books":[]}"#;
        let b = r#"{"taxons":[],"characters":[],"states":[{"id":1,"name":"B"}],"books":[]}"#;
        let diff = diff_json_strs(a, b).unwrap();
        println!("DIFF OUTPUT: {}", diff);
        assert!(diff.contains("\"states\":{\"added\":[],\"removed\":[],\"modified\":[{\"name\":{\"old\":\"A\",\"new\":\"B\"}}]}"));
    }

    #[test]
    fn test_diff_json_strs_missing_property() {
        let a = r#"{"taxons":[],"characters":[],"states":[]}"#;
        let b = r#"{"taxons":[],"characters":[],"states":[],"books":[]}"#;
        let err = diff_json_strs(a, b);
        assert!(err.is_err());
    }
}
