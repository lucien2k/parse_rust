//! Test cases for the parse_rust library
use parse_rust::*;
use std::collections::HashMap;

#[test]
fn test_basic_parse() {
    let r = parse("{} {}", "hello world").unwrap();
    assert_eq!(r.fixed, vec!["hello", "world"]);
}

#[test]
fn test_no_match() {
    assert!(parse("{}", "hello world").is_none());
}

#[test]
fn test_literal_braces() {
    let r = parse("{{hello}} {}", "{hello} world").unwrap();
    assert_eq!(r.fixed, vec!["world"]);
}

#[test]
fn test_multiple_fields() {
    let r = parse("{} {} {}", "a b c").unwrap();
    assert_eq!(r.fixed, vec!["a", "b", "c"]);
}

#[test]
fn test_named_fields() {
    let r = parse("{first} {second}", "hello world").unwrap();
    assert_eq!(r.named["first"], "hello");
    assert_eq!(r.named["second"], "world");
}

#[test]
fn test_mixed_fields() {
    let r = parse("{} {name} {}", "a world c").unwrap();
    assert_eq!(r.fixed, vec!["a", "world", "c"]);
    assert_eq!(r.named["name"], "world");
}

#[test]
fn test_search() {
    let r = search("Age: {}", "Name: John, Age: 25").unwrap();
    assert_eq!(r.fixed, vec!["25"]);
}

#[test]
fn test_findall() {
    let results = findall("{}", "a b c");
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].fixed, vec!["a"]);
    assert_eq!(results[1].fixed, vec!["b"]);
    assert_eq!(results[2].fixed, vec!["c"]);
}

#[test]
fn test_int_conversion() {
    let r = parse("{:d}", "123").unwrap();
    assert_eq!(r.fixed, vec!["123"]);
    let value = r.converted.get("0").unwrap();
    assert_eq!(value.downcast_ref::<i64>().unwrap(), &123);
}

#[test]
fn test_float_conversion() {
    let r = parse("{:f}", "123.45").unwrap();
    assert_eq!(r.fixed, vec!["123.45"]);
    let value = r.converted.get("0").unwrap();
    assert_eq!(value.downcast_ref::<f64>().unwrap(), &123.45);
}

#[test]
fn test_word_conversion() {
    let r = parse("{:w}", "hello123").unwrap();
    assert_eq!(r.fixed, vec!["hello123"]);
    let value = r.converted.get("0").unwrap();
    assert_eq!(value.downcast_ref::<String>().unwrap(), "hello123");
}

#[test]
fn test_mixed_types() {
    let r = parse("{:d} {:f} {:w}", "123 45.67 hello").unwrap();
    assert_eq!(r.fixed, vec!["123", "45.67", "hello"]);
    
    let int_value = r.converted.get("0").unwrap();
    assert_eq!(int_value.downcast_ref::<i64>().unwrap(), &123);
    
    let float_value = r.converted.get("1").unwrap();
    assert_eq!(float_value.downcast_ref::<f64>().unwrap(), &45.67);
    
    let word_value = r.converted.get("2").unwrap();
    assert_eq!(word_value.downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn test_named_types() {
    let r = parse("{age:d}", "25").unwrap();
    assert_eq!(r.fixed, vec!["25"]);
    let value = r.converted.get("age").unwrap();
    assert_eq!(value.downcast_ref::<i64>().unwrap(), &25);
}

#[test]
fn test_complex_field_names() {
    let r = parse("{person.name} {array[0]}", "John 123").unwrap();
    assert_eq!(r.fixed, vec!["John", "123"]);
    assert_eq!(r.named.get("person__name"), Some(&"John".to_string()));
    assert_eq!(r.named.get("array__0"), Some(&"123".to_string()));
}

#[test]
fn test_custom_type() {
    #[derive(Debug)]
    struct HexConverter;
    impl TypeConverter for HexConverter {
        fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
            i64::from_str_radix(s.trim_start_matches("0x"), 16)
                .map(|n| Box::new(n) as Box<dyn std::any::Any>)
                .map_err(|_| ParseError::TypeConversionFailed)
        }
        
        fn get_pattern(&self) -> Option<&str> {
            Some(r"0x[0-9a-fA-F]+")
        }
    }
    
    let mut extra_types = HashMap::new();
    extra_types.insert("hex".to_string(), Box::new(HexConverter) as Box<dyn TypeConverter>);
    
    let r = parse_with_types("{:hex}", "0x1F", extra_types).unwrap();
    assert_eq!(r.fixed, vec!["0x1F"]);
    let value = r.converted.get("0").unwrap();
    assert_eq!(value.downcast_ref::<i64>().unwrap(), &31);
}

#[test]
fn test_error_cases() {
    // Invalid format string
    assert!(matches!(Parser::new("{", false), Err(ParseError::InvalidFormat)));
    
    // Invalid type conversion
    let r = parse("{:d}", "abc");
    assert!(r.is_none());
}

#[test]
fn test_case_sensitivity() {
    // Case insensitive (default)
    let r = parse("HELLO {}", "hello world");
    assert!(r.is_some());
    
    // Case sensitive
    let parser = Parser::new("HELLO {}", true).unwrap();
    assert!(parser.parse("hello world").is_none());
    assert!(parser.parse("HELLO world").is_some());
}

#[test]
fn test_empty_patterns() {
    // Empty format string
    let r = parse("", "");
    assert!(r.is_some());
    assert!(r.unwrap().fixed.is_empty());
    
    // Empty input string
    let r = parse("{}", "");
    assert!(r.is_none());
}
