use crate::json::JsonValue;
use std::collections::HashSet;

pub fn diff_json_value<'a>(a: &JsonValue<'a>, b: &JsonValue<'a>, force_empty_array_diff: bool) -> Option<String> {
    use JsonValue::*;
    match (a, b) {
        (String(sa), String(sb)) => {
            if sa == sb {
                None
            } else {
                Some(format!("{{\"old\":{old},\"new\":{new}}}", old=json_escape(sa), new=json_escape(sb)))
            }
        }
        (Number(na), Number(nb)) => {
            if na == nb {
                None
            } else {
                Some(format!("{{\"old\":{},\"new\":{}}}", na, nb))
            }
        }
        (Boolean(ba), Boolean(bb)) => {
            if ba == bb {
                None
            } else {
                Some(format!("{{\"old\":{},\"new\":{}}}", ba, bb))
            }
        }
        (Null, Null) => None,
        (Array(va), Array(vb)) => {
            let mut added = Vec::new();
            let mut removed = Vec::new();
            let mut modified = Vec::new();
            let min_len = va.len().min(vb.len());
            for i in 0..min_len {
                if let Some(diff) = diff_json_value(&va[i], &vb[i], false) {
                    modified.push(diff);
                }
            }
            if vb.len() > va.len() {
                for be in &vb[va.len()..] {
                    added.push(json_value_to_string(be));
                }
            }
            if va.len() > vb.len() {
                for ae in &va[vb.len()..] {
                    removed.push(json_value_to_string(ae));
                }
            }
            let mut fields = Vec::new();
            if !added.is_empty() {
                fields.push(format!("\"added\":[{}]", added.join(",")));
            }
            if !removed.is_empty() {
                fields.push(format!("\"removed\":[{}]", removed.join(",")));
            }
            if !modified.is_empty() {
                fields.push(format!("\"modified\":[{}]", modified.join(",")));
            }
            // Toujours inclure added/removed même vides si force_empty_array_diff
            if force_empty_array_diff {
                if !fields.iter().any(|f| f.starts_with("\"added\":")) {
                    fields.insert(0, "\"added\":[]".to_string());
                }
                if !fields.iter().any(|f| f.starts_with("\"removed\":")) {
                    fields.insert(1, "\"removed\":[]".to_string());
                }
            }
            if fields.is_empty() {
                None
            } else {
                Some(format!("{{{}}}", fields.join(",")))
            }
        }
        (Object(oa), Object(ob)) => {
            let keys_a: HashSet<_> = oa.iter().map(|(k, _)| *k).collect();
            let keys_b: HashSet<_> = ob.iter().map(|(k, _)| *k).collect();
            let mut added = Vec::new();
            let mut removed = Vec::new();
            let mut modified = Vec::new();
            for k in keys_a.union(&keys_b) {
                let va = oa.iter().find(|(key, _)| key == k).map(|(_, v)| v);
                let vb = ob.iter().find(|(key, _)| key == k).map(|(_, v)| v);
                match (va, vb) {
                    (Some(_), None) => {
                        removed.push(format!("\"{}\":{}", escape_key(k), json_value_to_string(va.unwrap())));
                    }
                    (None, Some(_)) => {
                        added.push(format!("\"{}\":{}", escape_key(k), json_value_to_string(vb.unwrap())));
                    }
                    (Some(va), Some(vb)) => {
                        if let Some(diff) = diff_json_value(va, vb, false) {
                            modified.push(format!("\"{}\":{}", escape_key(k), diff));
                        }
                    }
                    (None, None) => {}
                }
            }
            let mut fields = Vec::new();
            if !added.is_empty() {
                fields.push(format!("\"added\":{{{}}}", added.join(",")));
            }
            if !removed.is_empty() {
                fields.push(format!("\"removed\":{{{}}}", removed.join(",")));
            }
            if !modified.is_empty() {
                fields.push(format!("\"modified\":{{{}}}", modified.join(",")));
            }
            if fields.is_empty() {
                None
            } else {
                Some(format!("{{{}}}", fields.join(",")))
            }
        }
        (a, b) => {
            if json_value_to_string(a) == json_value_to_string(b) {
                None
            } else {
                Some(format!("{{\"old\":{},\"new\":{}}}", json_value_to_string(a), json_value_to_string(b)))
            }
        }
    }
}

fn escape_key(key: &str) -> String {
    key.replace('"', "\\\"")
}

fn json_escape(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\\\"").replace('\n', "\\n").replace('\\', "\\\\"))
}

pub fn json_value_to_string<'a>(v: &JsonValue<'a>) -> String {
    use JsonValue::*;
    match v {
        String(s) => json_escape(s),
        Number(n) => n.to_string(),
        Boolean(b) => b.to_string(),
        Null => "null".to_string(),
        Array(arr) => format!("[{}]", arr.iter().map(json_value_to_string).collect::<Vec<_>>().join(",")),
        Object(obj) => {
            let mut map = Vec::new();
            for (k, v) in obj {
                map.push(format!("\"{}\":{}", escape_key(k), json_value_to_string(v)));
            }
            format!("{{{}}}", map.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse_json;

    fn diff_str(a: &str, b: &str, force: bool) -> Option<String> {
        let va = parse_json(a).unwrap();
        let vb = parse_json(b).unwrap();
        diff_json_value(&va, &vb, force)
    }

    #[test]
    fn test_diff_array_no_change_forced() {
        let a = "[1,2,3]";
        let b = "[1,2,3]";
        assert_eq!(diff_str(a, b, true), Some("{\"added\":[],\"removed\":[]}".to_string()));
    }

    #[test]
    fn test_diff_array_added_removed() {
        let a = "[1,2]";
        let b = "[1,2,3]";
        assert_eq!(diff_str(a, b, true), Some("{\"added\":[3],\"removed\":[]}".to_string()));
        let a = "[1,2,3]";
        let b = "[1,2]";
        assert_eq!(diff_str(a, b, true), Some("{\"added\":[],\"removed\":[3]}".to_string()));
    }

    #[test]
    fn test_diff_array_modified() {
        let a = "[1,2,3]";
        let b = "[1,4,3]";
        let expected = "{\"added\":[],\"removed\":[],\"modified\":[{\"old\":2,\"new\":4}]}";
        assert_eq!(diff_str(a, b, true), Some(expected.to_string()));
    }

    #[test]
    fn test_diff_object_added_removed() {
        let a = "{\"a\":1}";
        let b = "{\"a\":1,\"b\":2}";
        assert_eq!(diff_str(a, b, false), Some("{\"added\":{\"b\":2}}".to_string()));
        let a = "{\"a\":1,\"b\":2}";
        let b = "{\"a\":1}";
        assert_eq!(diff_str(a, b, false), Some("{\"removed\":{\"b\":2}}".to_string()));
    }

    #[test]
    fn test_diff_object_modified() {
        let a = "{\"a\":1,\"b\":2}";
        let b = "{\"a\":1,\"b\":3}";
        let expected = "{\"modified\":{\"b\":{\"old\":2,\"new\":3}}}";
        assert_eq!(diff_str(a, b, false), Some(expected.to_string()));
    }

    #[test]
    fn test_diff_scalar() {
        let a = "1";
        let b = "2";
        assert_eq!(diff_str(a, b, false), Some("{\"old\":1,\"new\":2}".to_string()));
        let a = "\"foo\"";
        let b = "\"bar\"";
        assert_eq!(diff_str(a, b, false), Some("{\"old\":\"foo\",\"new\":\"bar\"}".to_string()));
    }

    #[test]
    fn test_diff_array_no_change_not_forced() {
        let a = "[1,2,3]";
        let b = "[1,2,3]";
        assert_eq!(diff_str(a, b, false), None);
    }
}
