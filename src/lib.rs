use wasm_bindgen::prelude::*;

mod json;
mod jsondiff;
use crate::json::parse_json;
use crate::jsondiff::diff_json_value;

fn get_prop<'a>(obj: &'a crate::json::JsonValue<'a>, key: &str) -> Option<&'a crate::json::JsonValue<'a>> {
    match obj {
        crate::json::JsonValue::Object(fields) => fields.iter().find(|(k,_)| *k == key).map(|(_,v)| v),
        _ => None,
    }
}

pub fn diff_json_strs(old_json: &str, new_json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let old_val = parse_json(old_json).map_err(|e| format!("parse old: {}", e))?;
    let new_val = parse_json(new_json).map_err(|e| format!("parse new: {}", e))?;
    let mut result_fields = Vec::new();
    for prop in ["taxons", "characters", "states", "books"] {
        let old_p = get_prop(&old_val, prop).ok_or_else(|| format!("property '{}' missing in old JSON", prop))?;
        let new_p = get_prop(&new_val, prop).ok_or_else(|| format!("property '{}' missing in new JSON", prop))?;
        if let Some(diff) = diff_json_value(old_p, new_p, true) {
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