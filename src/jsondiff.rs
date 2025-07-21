use crate::buffer::ByteBuffer;
use crate::json::JsonValue;

pub fn diff_json_value<'a, B: ByteBuffer>(a: &JsonValue<'a>, b: &JsonValue<'a>, force_empty_array_diff: bool, flat_object_diff: bool, buf: &mut B) {
    match (a, b) {
        (JsonValue::String(sa), JsonValue::String(sb)) => {
            if sa != sb {
                buf.push_str("{\"old\":");
                buf.push_str(&json_escape(sa));
                buf.push_str(",\"new\":");
                buf.push_str(&json_escape(sb));
                buf.push(b'}');
            }
        }
        (JsonValue::Number(na), JsonValue::Number(nb)) => {
            if na != nb {
                buf.push_str("{\"old\":");
                buf.push_str(&na.to_string());
                buf.push_str(",\"new\":");
                buf.push_str(&nb.to_string());
                buf.push(b'}');
            }
        }
        (JsonValue::Boolean(ba), JsonValue::Boolean(bb)) => {
            if ba != bb {
                buf.push_str("{\"old\":");
                buf.push_str(&ba.to_string());
                buf.push_str(",\"new\":");
                buf.push_str(&bb.to_string());
                buf.push(b'}');
            }
        }
        (JsonValue::Null, JsonValue::Null) => {}
        (JsonValue::Array(va), JsonValue::Array(vb)) => {
            let min_len = va.len().min(vb.len());
            let mut has_diff = false;
            if va.len() != vb.len() {
                has_diff = true;
            }
            for i in 0..min_len {
                match (&va[i], &vb[i]) {
                    (JsonValue::Object(_), JsonValue::Object(_)) => {
                        let mut tmp = String::new();
                        diff_json_value(&va[i], &vb[i], false, true, &mut tmp);
                        if !tmp.is_empty() {
                            has_diff = true;
                        }
                    }
                    _ => {
                        if json_value_to_string(&va[i]) != json_value_to_string(&vb[i]) {
                            has_diff = true;
                        }
                    }
                }
            }
            if !has_diff && !force_empty_array_diff {
                return;
            }
            buf.push(b'{');
            buf.push_str("\"added\":[");
            if vb.len() > va.len() {
                for (i, be) in vb[va.len()..].iter().enumerate() {
                    if i > 0 { buf.push(b','); }
                    buf.push_str(&json_value_to_string(be));
                }
            }
            buf.push_str("]");
            buf.push_str(",\"removed\":[");
            if va.len() > vb.len() {
                for (i, ae) in va[vb.len()..].iter().enumerate() {
                    if i > 0 { buf.push(b','); }
                    buf.push_str(&json_value_to_string(ae));
                }
            }
            buf.push_str("]");
            let mut first_mod = true;
            for i in 0..min_len {
                match (&va[i], &vb[i]) {
                    (JsonValue::Object(_), JsonValue::Object(_)) => {
                        let mut tmp = String::new();
                        diff_json_value(&va[i], &vb[i], false, true, &mut tmp);
                        if !tmp.is_empty() {
                            if first_mod {
                                buf.push_str(",\"modified\":[");
                                first_mod = false;
                            } else {
                                buf.push(b',');
                            }
                            buf.push_str(&tmp);
                        }
                    }
                    _ => {
                        if json_value_to_string(&va[i]) != json_value_to_string(&vb[i]) {
                            if first_mod {
                                buf.push_str(",\"modified\":[");
                                first_mod = false;
                            } else {
                                buf.push(b',');
                            }
                            buf.push(b'{');
                            buf.push_str("\"old\":");
                            buf.push_str(&json_value_to_string(&va[i]));
                            buf.push_str(",\"new\":");
                            buf.push_str(&json_value_to_string(&vb[i]));
                            buf.push(b'}');
                        }
                    }
                }
            }
            if !first_mod {
                buf.push(b']');
            }
            buf.push(b'}');
        }
        (JsonValue::Object(oa), JsonValue::Object(ob)) => {
            let mut added = Vec::new();
            let mut removed = Vec::new();
            let mut modified = Vec::new();
            let mut tmp = String::new();
            for k in unique_sorted_keys(oa, ob) {
                let va = oa.iter().find(|(key, _)| key == &k).map(|(_, v)| v);
                let vb = ob.iter().find(|(key, _)| key == &k).map(|(_, v)| v);
                match (va, vb) {
                    (Some(_), None) => {
                        let mut s = String::new();
                        s.push('"');
                        s.push_str(&escape_key(k));
                        s.push_str("\":");
                        s.push_str(&json_value_to_string(va.unwrap()));
                        removed.push(s);
                    }
                    (None, Some(_)) => {
                        let mut s = String::new();
                        s.push('"');
                        s.push_str(&escape_key(k));
                        s.push_str("\":");
                        s.push_str(&json_value_to_string(vb.unwrap()));
                        added.push(s);
                    }
                    (Some(va), Some(vb)) => {
                        let mut sub_buf = String::new();
                        crate::jsondiff::diff_json_value(va, vb, false, false, &mut sub_buf);
                        if !sub_buf.is_empty() {
                            let mut s = String::new();
                            s.push('"');
                            s.push_str(&escape_key(k));
                            s.push_str("\":");
                            s.push_str(&sub_buf);
                            modified.push(s);
                        }
                    }
                    (None, None) => {}
                }
            }
            tmp.push('{');
            if !added.is_empty() {
                tmp.push_str("\"added\":{");
                tmp.push_str(&added.join(","));
                tmp.push('}');
            }
            if !removed.is_empty() {
                if !added.is_empty() { tmp.push(','); }
                tmp.push_str("\"removed\":{");
                tmp.push_str(&removed.join(","));
                tmp.push('}');
            }
            if !modified.is_empty() {
                if !added.is_empty() || !removed.is_empty() { tmp.push(','); }
                if flat_object_diff {
                    tmp.push_str(&modified.join(","));
                } else {
                    tmp.push_str("\"modified\":{");
                    tmp.push_str(&modified.join(","));
                    tmp.push('}');
                }
            }
            tmp.push('}');
            buf.push_str(&tmp);
        }
        (a, b) => {
            if json_value_to_string(a) != json_value_to_string(b) {
                buf.push_str("{\"old\":");
                buf.push_str(&json_value_to_string(a));
                buf.push_str(",\"new\":");
                buf.push_str(&json_value_to_string(b));
                buf.push(b'}');
            }
        }
    }
}

fn escape_key(key: &str) -> String {
    key.replace('"', "\\\"")
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn json_value_to_string<'a>(v: &JsonValue<'a>) -> String {
    match v {
        JsonValue::String(s) => json_escape(s),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Boolean(b) => b.to_string(),
        JsonValue::Null => "null".to_string(),
        JsonValue::Array(arr) => {
            let mut s = String::with_capacity(2 + arr.len() * 8);
            s.push('[');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 { s.push(','); }
                s.push_str(&json_value_to_string(v));
            }
            s.push(']');
            s
        },
        JsonValue::Object(obj) => {
            let mut map = Vec::with_capacity(obj.len());
            for (k, v) in obj {
                let mut s = String::with_capacity(k.len() + 3);
                s.push('"');
                s.push_str(&escape_key(k));
                s.push_str("\":");
                s.push_str(&json_value_to_string(v));
                map.push(s);
            }
            let mut s = String::with_capacity(2 + map.iter().map(|x| x.len()).sum::<usize>() + map.len());
            s.push('{');
            for (i, entry) in map.iter().enumerate() {
                if i > 0 { s.push(','); }
                s.push_str(entry);
            }
            s.push('}');
            s
        }
    }
}

fn unique_sorted_keys<'a>(oa: &'a [( &'a str, JsonValue<'a> )], ob: &'a [( &'a str, JsonValue<'a> )]) -> Vec<&'a str> {
    let mut keys: Vec<&'a str> = oa.iter().map(|(k,_)| *k).chain(ob.iter().map(|(k,_)| *k)).collect();
    keys.sort_unstable();
    keys.dedup();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse_json;

    fn diff_str(a: &str, b: &str, force: bool) -> String {
        let va = parse_json(a).unwrap();
        let vb = parse_json(b).unwrap();
        let mut buf = String::new();
        diff_json_value(&va, &vb, force, false, &mut buf);
        buf
    }

    #[test]
    fn test_diff_array_no_change_forced() {
        let a = "[1,2,3]";
        let b = "[1,2,3]";
        assert_eq!(diff_str(a, b, true), "{\"added\":[],\"removed\":[]}");
    }

    #[test]
    fn test_diff_array_added_removed() {
        let a = "[1,2]";
        let b = "[1,2,3]";
        assert_eq!(diff_str(a, b, true), "{\"added\":[3],\"removed\":[]}");
        let a = "[1,2,3]";
        let b = "[1,2]";
        assert_eq!(diff_str(a, b, true), "{\"added\":[],\"removed\":[3]}");
    }

    #[test]
    fn test_diff_array_modified() {
        let a = "[1,2,3]";
        let b = "[1,4,3]";
        let expected = "{\"added\":[],\"removed\":[],\"modified\":[{\"old\":2,\"new\":4}]}";
        assert_eq!(diff_str(a, b, true), expected);
    }

    #[test]
    fn test_diff_object_added_removed() {
        let a = "{\"a\":1}";
        let b = "{\"a\":1,\"b\":2}";
        assert_eq!(diff_str(a, b, false), "{\"added\":{\"b\":2}}");
        let a = "{\"a\":1,\"b\":2}";
        let b = "{\"a\":1}";
        assert_eq!(diff_str(a, b, false), "{\"removed\":{\"b\":2}}");
    }

    #[test]
    fn test_diff_object_modified() {
        let a = "{\"a\":1,\"b\":2}";
        let b = "{\"a\":1,\"b\":3}";
        let expected = "{\"modified\":{\"b\":{\"old\":2,\"new\":3}}}";
        assert_eq!(diff_str(a, b, false), expected);
    }

    #[test]
    fn test_diff_scalar() {
        let a = "1";
        let b = "2";
        assert_eq!(diff_str(a, b, false), "{\"old\":1,\"new\":2}");
        let a = "\"foo\"";
        let b = "\"bar\"";
        assert_eq!(diff_str(a, b, false), "{\"old\":\"foo\",\"new\":\"bar\"}");
    }

    #[test]
    fn test_diff_array_no_change_not_forced() {
        let a = "[1,2,3]";
        let b = "[1,2,3]";
        assert_eq!(diff_str(a, b, false), "");
    }
}
