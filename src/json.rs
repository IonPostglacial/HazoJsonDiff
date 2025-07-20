#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct Token {
    start: usize,
    end: usize,
    token_type: TokenType,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut i = 0;
    let src = input.as_bytes();

    while i < src.len() {
        let c = src[i];
        match c {
            b'"' if !in_string => {
                in_string = true;
                start = i;
            }
            b'"' if in_string => {
                tokens.push(Token {
                    start,
                    end: i + 1,
                    token_type: TokenType::String,
                });
                in_string = false;
                start = i + 1;
            }
            b'{' if !in_string => {
                tokens.push(Token {
                    start: i,
                    end: i + 1,
                    token_type: TokenType::ObjectStart,
                });
                start = i + 1;
            }
            b'}' if !in_string => {
                tokens.push(Token {
                    start: i,
                    end: i + 1,
                    token_type: TokenType::ObjectEnd,
                });
                start = i + 1;
            }
            b'[' if !in_string => {
                tokens.push(Token {
                    start: i,
                    end: i + 1,
                    token_type: TokenType::ArrayStart,
                });
                start = i + 1;
            }
            b']' if !in_string => {
                tokens.push(Token {
                    start: i,
                    end: i + 1,
                    token_type: TokenType::ArrayEnd,
                });
                start = i + 1;
            }
            b',' if !in_string => {
                tokens.push(Token {
                    start: i,
                    end: i + 1,
                    token_type: TokenType::Comma,
                });
                start = i + 1;
            }
            b':' if !in_string => {
                tokens.push(Token {
                    start: i,
                    end: i + 1,
                    token_type: TokenType::Colon,
                });
                start = i + 1;
            }
            b't' if !in_string && i + 3 < src.len() && &src[i..i + 4] == b"true" => {
                tokens.push(Token {
                    start: i,
                    end: i + 4,
                    token_type: TokenType::True,
                });
                i += 3;
                start = i + 1;
            }
            b'f' if !in_string && i + 4 < src.len() && &src[i..i + 5] == b"false" => {
                tokens.push(Token {
                    start: i,
                    end: i + 5,
                    token_type: TokenType::False,
                });
                i += 4;
                start = i + 1;
            }
            b'n' if !in_string && i + 3 < src.len() && &src[i..i + 4] == b"null" => {
                tokens.push(Token {
                    start: i,
                    end: i + 4,
                    token_type: TokenType::Null,
                });
                i += 3;
                start = i + 1;
            }
            _ if !in_string && (c.is_ascii_digit() || c == b'-') => {
                let num_start = i;
                while i + 1 < src.len() && (src[i + 1].is_ascii_digit() || src[i + 1] == b'.') {
                    i += 1;
                }
                tokens.push(Token {
                    start: num_start,
                    end: i + 1,
                    token_type: TokenType::Number,
                });
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    tokens
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

#[derive(Debug)]
pub enum JsonParserErrorType<'a> {
    InvalidToken(Token),
    InvalidNumber(&'a str),
    InvalidStructure(String),
}

#[derive(Debug)]
pub struct JsonParserError<'a> {
    error_type: JsonParserErrorType<'a>,
}

pub fn parse_json<'a>(input: &'a str) -> Result<JsonValue<'a>, JsonParserError<'a>> {
    let tokens = tokenize(input);
    let mut stack: Vec<JsonValue<'a>> = Vec::new();

    for token in tokens {
        match token.token_type {
            TokenType::String => {
                let mut s = &input[token.start..token.end];
                if s.starts_with('"') {
                    s = &s[1..];
                }
                if s.ends_with('"') && s.len() > 0 {
                    s = &s[..s.len()-1];
                }
                stack.push(JsonValue::String(s));
            }
            TokenType::Number => {
                let number_str = &input[token.start..token.end];
                if let Ok(num) = number_str.parse::<f64>() {
                    stack.push(JsonValue::Number(num));
                } else {
                    return Err(JsonParserError {
                        error_type: JsonParserErrorType::InvalidNumber(number_str),
                    });
                }
            }
            TokenType::True => stack.push(JsonValue::Boolean(true)),
            TokenType::False => stack.push(JsonValue::Boolean(false)),
            TokenType::Null => stack.push(JsonValue::Null),
            TokenType::Comma | TokenType::Colon => continue,
            TokenType::ArrayStart => {
                stack.push(JsonValue::Array(Vec::new()));
            }
            TokenType::ArrayEnd => {
                let mut array = Vec::new();
                while let Some(val) = stack.pop() {
                    match val {
                        JsonValue::Array(_) => break, // Marqueur d'ouverture
                        _ => array.push(val),
                    }
                }
                array.reverse();
                stack.push(JsonValue::Array(array));
            }
            TokenType::ObjectStart => {
                stack.push(JsonValue::Object(Vec::new()));
            }
            TokenType::ObjectEnd => {
                let mut temp_vals = Vec::new();
                while let Some(val) = stack.pop() {
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
                                error_type: JsonParserErrorType::InvalidStructure("Object key is not a string".to_string()),
                            });
                        }
                    }
                }
                pairs.reverse();
                stack.push(JsonValue::Object(pairs));
            }
        }
    }

    if stack.len() == 1 {
        Ok(stack.pop().unwrap())
    } else {
        Err(JsonParserError {
            error_type: JsonParserErrorType::InvalidStructure("Invalid JSON structure".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = r#"{"key": "value", "number": 123, "boolean": true, "null_value": null}"#;
        let tokens = tokenize(input);
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
                JsonParserErrorType::InvalidStructure(msg) => {
                    assert_eq!(msg, "Invalid JSON structure");
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