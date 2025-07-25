pub use crate::errors::{JsonDiffError, JsonDiffErrorType};

#[cfg_attr(test, derive(Debug))]
#[derive(PartialEq)]
enum TokenType {
    String,
    Number,
    True,
    False,
    Null,
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Comma,
    Colon,
}

#[cfg_attr(test, derive(Debug))]
pub struct Token {
    start: usize,
    end: usize,
    token_type: TokenType,
}

struct Tokenizer<'a> {
    src: &'a [u8],
    i: usize,
    start: usize,
    in_string: bool,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            src: input.as_bytes(),
            i: 0,
            start: 0,
            in_string: false,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        while self.i < self.src.len() {
            let c = self.src[self.i];
            match c {
                b'\\' if self.in_string => {
                    self.i += 2;
                    continue;
                }
                b'"' if !self.in_string => {
                    self.in_string = true;
                    self.start = self.i;
                }
                b'"' if self.in_string => {
                    let token = Token {
                        start: self.start,
                        end: self.i + 1,
                        token_type: TokenType::String,
                    };
                    self.in_string = false;
                    self.start = self.i + 1;
                    self.i += 1;
                    return Some(token);
                }
                b'{' if !self.in_string => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 1,
                        token_type: TokenType::ObjectStart,
                    };
                    self.start = self.i + 1;
                    self.i += 1;
                    return Some(token);
                }
                b'}' if !self.in_string => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 1,
                        token_type: TokenType::ObjectEnd,
                    };
                    self.start = self.i + 1;
                    self.i += 1;
                    return Some(token);
                }
                b'[' if !self.in_string => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 1,
                        token_type: TokenType::ArrayStart,
                    };
                    self.start = self.i + 1;
                    self.i += 1;
                    return Some(token);
                }
                b']' if !self.in_string => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 1,
                        token_type: TokenType::ArrayEnd,
                    };
                    self.start = self.i + 1;
                    self.i += 1;
                    return Some(token);
                }
                b',' if !self.in_string => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 1,
                        token_type: TokenType::Comma,
                    };
                    self.start = self.i + 1;
                    self.i += 1;
                    return Some(token);
                }
                b':' if !self.in_string => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 1,
                        token_type: TokenType::Colon,
                    };
                    self.start = self.i + 1;
                    self.i += 1;
                    return Some(token);
                }
                b't' if !self.in_string && self.i + 3 < self.src.len() && &self.src[self.i..self.i + 4] == b"true" => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 4,
                        token_type: TokenType::True,
                    };
                    self.i += 4;
                    self.start = self.i;
                    return Some(token);
                }
                b'f' if !self.in_string && self.i + 4 < self.src.len() && &self.src[self.i..self.i + 5] == b"false" => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 5,
                        token_type: TokenType::False,
                    };
                    self.i += 5;
                    self.start = self.i;
                    return Some(token);
                }
                b'n' if !self.in_string && self.i + 3 < self.src.len() && &self.src[self.i..self.i + 4] == b"null" => {
                    let token = Token {
                        start: self.i,
                        end: self.i + 4,
                        token_type: TokenType::Null,
                    };
                    self.i += 4;
                    self.start = self.i;
                    return Some(token);
                }
                _ if !self.in_string && (c.is_ascii_digit() || c == b'-') => {
                    let num_start = self.i;
                    while self.i + 1 < self.src.len() && (self.src[self.i + 1].is_ascii_digit() || self.src[self.i + 1] == b'.') {
                        self.i += 1;
                    }
                    let token = Token {
                        start: num_start,
                        end: self.i + 1,
                        token_type: TokenType::Number,
                    };
                    self.i += 1;
                    self.start = self.i;
                    return Some(token);
                }
                _ => {}
            }
            self.i += 1;
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue<'a> {
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<JsonValue<'a>>),
    Object(Vec<(&'a str, JsonValue<'a>)>),
}

impl std::fmt::Display for JsonDiffErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonDiffErrorType::InvalidStructureObjectKey => write!(f, "Invalid object key: expected string key in object"),
            JsonDiffErrorType::PropertyMissing => write!(f, "Property missing"),
            JsonDiffErrorType::InvalidStructureUnclosed => write!(f, "Invalid structure: unclosed object or array"),
            JsonDiffErrorType::InvalidStructureUnexpectedToken => write!(f, "Invalid structure: unexpected token"),
            JsonDiffErrorType::InvalidStructureInvalidNumber => write!(f, "Invalid structure: invalid number"),
        }
    }
}

impl std::fmt::Display for JsonDiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_type)
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, JsonDiffError> {
    let tokenizer = Tokenizer::new(input);
    let mut stack_vec: Vec<JsonValue> = Vec::new();

    for token in tokenizer {
        match token.token_type {
            TokenType::String => {
                let mut s = &input[token.start..token.end];
                if s.starts_with('"') {
                    s = &s[1..];
                }
                if s.ends_with('"') && s.len() > 0 {
                    s = &s[..s.len()-1];
                }
                stack_vec.push(JsonValue::String(s));
            }
            TokenType::Number => {
                let number_str = &input[token.start..token.end];
                if let Ok(num) = number_str.parse::<f64>() {
                    stack_vec.push(JsonValue::Number(num));
                } else {
                    return Err(JsonDiffError {
                        error_type: JsonDiffErrorType::InvalidStructureInvalidNumber,
                    });
                }
            }
            TokenType::True => stack_vec.push(JsonValue::Boolean(true)),
            TokenType::False => stack_vec.push(JsonValue::Boolean(false)),
            TokenType::Null => stack_vec.push(JsonValue::Null),
            TokenType::Comma | TokenType::Colon => continue,
            TokenType::ArrayStart => {
                stack_vec.push(JsonValue::Array(Vec::new()));
            }
            TokenType::ArrayEnd => {
                let mut array = Vec::new();
                while let Some(val) = stack_vec.pop() {
                    match val {
                        JsonValue::Array(_) => break, // Marqueur d'ouverture
                        _ => array.push(val),
                    }
                }
                array.reverse();
                stack_vec.push(JsonValue::Array(array));
            }
            TokenType::ObjectStart => {
                stack_vec.push(JsonValue::Object(Vec::new()));
            }
            TokenType::ObjectEnd => {
                let mut temp_vals = Vec::new();
                while let Some(val) = stack_vec.pop() {
                    match val {
                        JsonValue::Object(ref obj) if obj.is_empty() => {
                            break;
                        }
                        _ => {
                            temp_vals.push(val);
                        }
                    }
                }
                temp_vals.reverse();
                let mut pairs = Vec::new();
                let mut i = 0;
                while i + 1 < temp_vals.len() {
                    let key = match &temp_vals[i] {
                        JsonValue::String(s) => *s,
                        _ => {
                            return Err(JsonDiffError {
                                error_type: JsonDiffErrorType::InvalidStructureObjectKey,
                            });
                        }
                    };
                    let value = temp_vals[i + 1].clone();
                    pairs.push((key, value));
                    i += 2;
                }
                stack_vec.push(JsonValue::Object(pairs));
            }
        }
    }

    if stack_vec.len() == 1 {
        Ok(stack_vec.pop().unwrap())
    } else {
        Err(JsonDiffError {
            error_type: JsonDiffErrorType::InvalidStructureUnclosed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = r#"{"key": "value", "number": 123, "boolean": true, "null_value": null}"#;
        let tokens: Vec<_> = Tokenizer::new(input).collect();
        assert_eq!(tokens.len(), 17);
        assert_eq!(tokens[0].token_type, TokenType::ObjectStart);
        assert_eq!(tokens[1].token_type, TokenType::String);
        assert_eq!(tokens[2].token_type, TokenType::Colon);
        assert_eq!(tokens[3].token_type, TokenType::String);
        assert_eq!(tokens[4].token_type, TokenType::Comma);
        assert_eq!(tokens[5].token_type, TokenType::String);
        assert_eq!(tokens[6].token_type, TokenType::Colon);
        assert_eq!(tokens[7].token_type, TokenType::Number);
        assert_eq!(tokens[8].token_type, TokenType::Comma);
        assert_eq!(tokens[9].token_type, TokenType::String);
        assert_eq!(tokens[10].token_type, TokenType::Colon);
        assert_eq!(tokens[11].token_type, TokenType::True);
        assert_eq!(tokens[12].token_type, TokenType::Comma);
        assert_eq!(tokens[13].token_type, TokenType::String);
        assert_eq!(tokens[14].token_type, TokenType::Colon);
        assert_eq!(tokens[15].token_type, TokenType::Null);
        assert_eq!(tokens[16].token_type, TokenType::ObjectEnd);
    }

    #[test]
    fn test_tokenize_nested() {
        let input = r#"{"outer": {"inner": "value"}}"#;
        let tokens: Vec<_> = Tokenizer::new(input).collect();
        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0].token_type, TokenType::ObjectStart);
        assert_eq!(tokens[1].token_type, TokenType::String);
        assert_eq!(tokens[2].token_type, TokenType::Colon);
        assert_eq!(tokens[3].token_type, TokenType::ObjectStart);
        assert_eq!(tokens[4].token_type, TokenType::String);
        assert_eq!(tokens[5].token_type, TokenType::Colon);
        assert_eq!(tokens[6].token_type, TokenType::String);
        assert_eq!(tokens[7].token_type, TokenType::ObjectEnd);
        assert_eq!(tokens[8].token_type, TokenType::ObjectEnd);
    }

    #[test]
    fn test_parse_json_atoms() {
        let input = r#""string_value""#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        if let JsonValue::String(value) = json_value.unwrap() {
            assert_eq!(value, "string_value");
        } else {
            panic!("Expected JSON value to be a string");
        }

        let input = r#"123.456"#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        if let JsonValue::Number(num) = json_value.unwrap() {
            assert_eq!(num, 123.456);
        } else {
            panic!("Expected JSON value to be a number");
        }

        let input = r#"true"#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        if let JsonValue::Boolean(boolean) = json_value.unwrap() {
            assert!(boolean);
        } else {
            panic!("Expected JSON value to be a boolean");
        }

        let input = r#"false"#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        if let JsonValue::Boolean(boolean) = json_value.unwrap() {
            assert!(!boolean);
        } else {
            panic!("Expected JSON value to be a boolean");
        }

        let input = r#"null"#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        if let JsonValue::Null = json_value.unwrap() {
            // Expected null value
        } else {
            panic!("Expected JSON value to be null");
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let input = r#"[]"#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        if let JsonValue::Array(array) = json_value.unwrap() {
            assert!(array.is_empty());
        } else {
            panic!("Expected JSON value to be an empty array");
        }
    }

    #[test]
    fn test_parse_single_element_array() {
        let input = r#"[1]"#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        if let JsonValue::Array(array) = json_value.unwrap() {
            assert_eq!(array.len(), 1);
            if let JsonValue::Number(value) = &array[0] {
                assert_eq!(*value, 1.0);
            } else {
                panic!("Expected single element to be a number");
            }
        } else {
            panic!("Expected JSON value to be an array");
        }
    }

    #[test]
    fn test_parse_json_array() {
        let input = r#"[1, "two", true, null]"#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        if let JsonValue::Array(array) = json_value.unwrap() {
            assert_eq!(array.len(), 4);
            if let JsonValue::Number(num) = &array[0] {
                assert_eq!(*num, 1.0);
            } else {
                panic!("Expected first element to be a number");
            }
            if let JsonValue::String(value) = &array[1] {
                assert_eq!(*value, "two");
            } else {
                panic!("Expected second element to be a string");
            }
            if let JsonValue::Boolean(boolean) = &array[2] {
                assert!(*boolean);
            } else {
                panic!("Expected third element to be a boolean");
            }
            if JsonValue::Null != array[3] {
                panic!("Expected fourth element to be null");
            } else {
            }
        } else {
            panic!("Expected JSON value to be an array");
        }
    }

    #[test]
    fn test_parse_json() {
        let input = r#"{"key": "value", "number": 123, "boolean": true, "null_value": null}"#;
        let json_value = parse_json(input);
        assert!(json_value.is_ok());
        let json_value = json_value.unwrap();
        if let JsonValue::Object(obj) = json_value {
            assert_eq!(obj.len(), 4);
            assert_eq!(obj[0].0, "key");
            if let JsonValue::String(value) = &obj[0].1 {
                assert_eq!(*value, "value");
            } else {
                panic!("Expected string value for 'key'");
            }
            assert_eq!(obj[1].0, "number");
            if let JsonValue::Number(num) = &obj[1].1 {
                assert_eq!(*num, 123.0);
            } else {
                panic!("Expected number value for 'number'");
            }
            assert_eq!(obj[2].0, "boolean");
            if let JsonValue::Boolean(boolean) = &obj[2].1 {
                assert!(*boolean);
            } else {
                panic!("Expected boolean value for 'boolean'");
            }
            assert_eq!(obj[3].0, "null_value");
            if let JsonValue::Null = &obj[3].1 {
                // Expected null value
            } else {
                panic!("Expected null value for 'null_value'");
            }
        } else {
            panic!("Expected JSON value to be an object");
        }
    }

    #[test]
    fn test_invalid_json() {
        let input = r#"{"key": "value", "number": 123, "boolean": true, "null_value": null"#;
        let json_value = parse_json(input);
        assert!(json_value.is_err());
        if let Err(JsonDiffError { error_type }) = json_value {
            match error_type {
                JsonDiffErrorType::InvalidStructureUnclosed => {
                    // Erreur attendue pour la structure invalide
                }
                _ => panic!("Expected InvalidStructure error type"),
            }
        } else {
            panic!("Expected an error for invalid JSON");
        }
    }

    #[test]
    fn test_parse_realistic_json() {
        let input = r#"{
  "id": "Antremaplante61",
  "taxons": [
    {
      "id": "t1",
      "path": [
        "t0"
      ],
      "name": "Acanthaceae",
      "nameEN": "",
      "nameCN": "爵床科",
      "vernacularName": "",
      "detail": "",
      "children": []
    }
  ]
}"#;
        let json_value = parse_json(input);
        if json_value.is_err() {
            println!("Erreur de parsing: {:?}", json_value);
        }
        let json_value = match json_value {
            Ok(v) => v,
            Err(_) => panic!("Parsing failed"),
        };
        let obj = match json_value {
            JsonValue::Object(obj) => obj,
            _ => panic!("Expected JSON value to be an object"),
        };
        assert_eq!(obj[0].0, "id");
        match &obj[0].1 {
            JsonValue::String(val) => assert_eq!(*val, "Antremaplante61"),
            _ => panic!("Expected string value for 'id'"),
        }
        assert_eq!(obj[1].0, "taxons");
        let taxons = match &obj[1].1 {
            JsonValue::Array(taxons) => taxons,
            _ => panic!("Expected array for 'taxons'"),
        };
        assert!(!taxons.is_empty());
        let taxon = match &taxons[0] {
            JsonValue::Object(taxon) => taxon,
            _ => panic!("Expected object in taxons array"),
        };
        assert_eq!(taxon[0].0, "id");
        match &taxon[0].1 {
            JsonValue::String(val) => assert_eq!(*val, "t1"),
            _ => panic!("Expected string value for 'id' in taxon"),
        }
    }

    #[test]
    fn test_parse_quoted_string() {
        let input = r#""\"hello\"""#;

        let result = parse_json(input);
        assert!(result.is_ok(), "Expected JSON to parse successfully, but got error: {:?}", result);

        if let Ok(JsonValue::String(value)) = result {
            assert_eq!(value, "\\\"hello\\\"");
        } else {
            panic!("Expected JSON value to be a string");
        }
    }

    #[test]
    fn test_parse_nested_object() {
        let input = r#"{"outer": {"inner": "value"}}"#;
        let result = parse_json(input);
        assert!(result.is_ok(), "Expected JSON to parse successfully, but got error: {:?}", result);

        if let Ok(JsonValue::Object(obj)) = result {
            assert_eq!(obj.len(), 1);
            assert_eq!(obj[0].0, "outer");
            if let JsonValue::Object(inner_obj) = &obj[0].1 {
                assert_eq!(inner_obj.len(), 1);
                if let JsonValue::String(value) = &inner_obj[0].1 {
                    assert_eq!(*value, "value");
                } else {
                    panic!("Expected string value for 'inner'");
                }
            } else {
                panic!("Expected object value for 'outer'");
            }
        } else {
            panic!("Expected JSON value to be an object");
        }
    }

    #[test]
    fn test_parse_array_with_objects() {
        let input = r#"[{"key": "value"}, {"key2": "value2"}]"#;
        let result = parse_json(input);
        assert!(result.is_ok(), "Expected JSON to parse successfully, but got error: {:?}", result);

        if let Ok(JsonValue::Array(array)) = result {
            assert_eq!(array.len(), 2);
            if let JsonValue::Object(obj1) = &array[0] {
                assert_eq!(obj1.len(), 1);
                assert_eq!(obj1[0].0, "key");
                if let JsonValue::String(value) = &obj1[0].1 {
                    assert_eq!(*value, "value");
                } else {
                    panic!("Expected string value for 'key'");
                }
            } else {
                panic!("Expected first element to be an object");
            }

            if let JsonValue::Object(obj2) = &array[1] {
                assert_eq!(obj2.len(), 1);
                assert_eq!(obj2[0].0, "key2");
                if let JsonValue::String(value) = &obj2[0].1 {
                    assert_eq!(*value, "value2");
                } else {
                    panic!("Expected string value for 'key2'");
                }
            } else {
                panic!("Expected second element to be an object");
            }
        } else {
            panic!("Expected JSON value to be an array");
        }
    }
}