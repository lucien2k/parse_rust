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
    let p = Parser::new("{:d}", true).unwrap();
    let r = p.parse("123").unwrap();
    let value: &i64 = r.get(0).unwrap();
    assert_eq!(value, &123);
}

#[test]
fn test_float_conversion() {
    let p = Parser::new("{:f}", true).unwrap();
    let r = p.parse("123.45").unwrap();
    let value: &f64 = r.get(0).unwrap();
    assert_eq!(value, &123.45);
}

#[test]
fn test_word_conversion() {
    let p = Parser::new("{:w}", true).unwrap();
    let r = p.parse("hello123").unwrap();
    let value: &String = r.get(0).unwrap();
    assert_eq!(value, "hello123");
}

#[test]
fn test_mixed_types() {
    let p = Parser::new("{:d} {:f} {:w}", true).unwrap();
    let r = p.parse("123 45.67 hello").unwrap();
    
    let int_value: &i64 = r.get(0).unwrap();
    assert_eq!(int_value, &123);
    
    let float_value: &f64 = r.get(1).unwrap();
    assert_eq!(float_value, &45.67);
    
    let word_value: &String = r.get(2).unwrap();
    assert_eq!(word_value, "hello");
}

#[test]
fn test_named_types() {
    let p = Parser::new("{age:d}", true).unwrap();
    let r = p.parse("25").unwrap();
    let value: &i64 = r.get(0).unwrap();
    assert_eq!(value, &25);
}

#[test]
fn test_custom_type() {
    struct CustomType {
        value: i64,
    }

    #[derive(Debug)]
    struct CustomConverter;
    impl TypeConverter for CustomConverter {
        fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
            s.parse::<i64>()
                .map(|n| Box::new(CustomType { value: n }) as Box<dyn std::any::Any>)
                .map_err(|_| ParseError::TypeConversionFailed)
        }

        fn get_pattern(&self) -> Option<&str> {
            Some(r"\d+")
        }
    }

    let mut extra_types = HashMap::new();
    extra_types.insert("custom".to_string(), Box::new(CustomConverter) as Box<dyn TypeConverter>);
    
    let p = Parser::new_with_types("{:custom}", true, extra_types).unwrap();
    let r = p.parse("31").unwrap();
    let value: &CustomType = r.get(0).unwrap();
    assert_eq!(value.value, 31);
}

#[test]
fn test_complex_field_names() {
    let r = parse("{person.name} {array[0]}", "John 123").unwrap();
    assert_eq!(r.fixed, vec!["John", "123"]);
    assert_eq!(r.named.get("person__name"), Some(&"John".to_string()));
    assert_eq!(r.named.get("array__0"), Some(&"123".to_string()));
}

#[test]
fn test_datetime_conversion() {
    // Test datetime parsing
    let p = Parser::new("{:datetime}", true).unwrap();
    let result = p.parse("2024-12-27 19:57:55").unwrap();
    let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
    assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2024-12-27 19:57:55");

    // Test with T separator
    let result = p.parse("2024-12-27T19:57:55").unwrap();
    let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
    assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2024-12-27 19:57:55");

    // Test with Z suffix
    let result = p.parse("2024-12-27T19:57:55Z").unwrap();
    let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
    assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2024-12-27 19:57:55");
}

#[test]
fn test_date_conversion() {
    let p = Parser::new("{:date}", true).unwrap();
    
    // Test ISO format
    let result = p.parse("2024-12-27").unwrap();
    let date: &chrono::NaiveDate = result.get(0).unwrap();
    assert_eq!(date.format("%Y-%m-%d").to_string(), "2024-12-27");

    // Test slash format
    let result = p.parse("27/12/2024").unwrap();
    let date: &chrono::NaiveDate = result.get(0).unwrap();
    assert_eq!(date.format("%Y-%m-%d").to_string(), "2024-12-27");

    // Test named month format
    let result = p.parse("27-Dec-2024").unwrap();
    let date: &chrono::NaiveDate = result.get(0).unwrap();
    assert_eq!(date.format("%Y-%m-%d").to_string(), "2024-12-27");

    // Test compact format
    let result = p.parse("20241227").unwrap();
    let date: &chrono::NaiveDate = result.get(0).unwrap();
    assert_eq!(date.format("%Y-%m-%d").to_string(), "2024-12-27");
}

#[test]
fn test_time_conversion() {
    let p = Parser::new("{:time}", true).unwrap();
    
    // Test 24-hour format with seconds
    let result = p.parse("19:57:55").unwrap();
    let time: &chrono::NaiveTime = result.get(0).unwrap();
    assert_eq!(time.format("%H:%M:%S").to_string(), "19:57:55");

    // Test 24-hour format without seconds
    let result = p.parse("19:57").unwrap();
    let time: &chrono::NaiveTime = result.get(0).unwrap();
    assert_eq!(time.format("%H:%M:%S").to_string(), "19:57:00");

    // Test 12-hour format with seconds
    let result = p.parse("07:57:55 PM").unwrap();
    let time: &chrono::NaiveTime = result.get(0).unwrap();
    assert_eq!(time.format("%H:%M:%S").to_string(), "19:57:55");

    // Test 12-hour format without seconds
    let result = p.parse("07:57 PM").unwrap();
    let time: &chrono::NaiveTime = result.get(0).unwrap();
    assert_eq!(time.format("%H:%M:%S").to_string(), "19:57:00");
}

#[test]
fn test_datetime_errors() {
    let p = Parser::new("{:datetime}", true).unwrap();
    
    // Invalid date
    assert!(p.parse("2024-13-27 19:57:55").is_none());
    
    // Invalid time
    assert!(p.parse("2024-12-27 25:57:55").is_none());
    
    // Invalid format
    assert!(p.parse("2024/12/27 19:57:55").is_none());
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
