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

#[derive(Debug, Clone)]
pub enum JsonValue<'a> {
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<JsonValue<'a>>),
    Object(Vec<(&'a str, JsonValue<'a>)>),
}

#[cfg_attr(test, derive(Debug))]
pub enum JsonParserErrorType {
    InvalidStructureObjectKey,
    InvalidStructureGeneral,
}

#[cfg_attr(test, derive(Debug))]
pub struct JsonParserError {
    #[allow(dead_code)]
    error_type: JsonParserErrorType,
}

impl std::fmt::Display for JsonParserErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonParserErrorType::InvalidStructureObjectKey => write!(f, "Invalid object key: expected string key in object"),
            JsonParserErrorType::InvalidStructureGeneral => write!(f, "Invalid JSON structure"),
        }
    }
}

impl std::fmt::Display for JsonParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_type)
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, JsonParserError> {
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
                    return Err(JsonParserError {
                        error_type: JsonParserErrorType::InvalidStructureGeneral,
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
                        JsonValue::Object(_) => break, // Marqueur d'ouverture
                        _ => temp_vals.push(val),
                    }
                }
                temp_vals.reverse();
                let mut pairs = Vec::new();
                while temp_vals.len() >= 2 {
                    let value = temp_vals.pop().unwrap();
                    match temp_vals.pop().unwrap() {
                        JsonValue::String(key) => pairs.push((key, value)),
                        _ => {
                            return Err(JsonParserError {
                                error_type: JsonParserErrorType::InvalidStructureObjectKey,
                            });
                        }
                    }
                }
                pairs.reverse();
                stack_vec.push(JsonValue::Object(pairs));
            }
        }
    }

    if stack_vec.len() == 1 {
        Ok(stack_vec.pop().unwrap())
    } else {
        Err(JsonParserError {
            error_type: JsonParserErrorType::InvalidStructureGeneral,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = r#"{\"key\": \"value\", \"number\": 123, \"boolean\": true, \"null_value\": null}"#;
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
            if let JsonValue::Null = &array[3] {
                // Expected fourth element to be null
            } else {
                panic!("Expected fourth element to be null");
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
        if let Err(JsonParserError { error_type }) = json_value {
            match error_type {
                JsonParserErrorType::InvalidStructureGeneral => {
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
        assert!(json_value.is_ok());
        if let JsonValue::Object(obj) = json_value.unwrap() {
            assert_eq!(obj[0].0, "id");
            if let JsonValue::String(val) = &obj[0].1 {
                assert_eq!(*val, "Antremaplante61");
            } else {
                panic!("Expected string value for 'id'");
            }
            assert_eq!(obj[1].0, "taxons");
            if let JsonValue::Array(taxons) = &obj[1].1 {
                assert!(!taxons.is_empty());
                if let JsonValue::Object(taxon) = &taxons[0] {
                    assert_eq!(taxon[0].0, "id");
                    if let JsonValue::String(val) = &taxon[0].1 {
                        assert_eq!(*val, "t1");
                    } else {
                        panic!("Expected string value for 'id' in taxon");
                    }
                } else {
                    panic!("Expected object in taxons array");
                }
            } else {
                panic!("Expected array for 'taxons'");
            }
        } else {
            panic!("Expected JSON value to be an object");
        }
    }
}